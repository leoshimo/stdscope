use std::path::PathBuf;
use std::process::Stdio;
use anyhow::{Result, Context};
use clap::Parser;
use tokio::io::{AsyncWriteExt, BufWriter, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::select;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Write captured output to file (default: stdout)
    #[arg(short = 'o', long = "output", conflicts_with = "stderr")]
    output: Option<PathBuf>,

    /// Write captured output to stderr
    #[arg(short = 'e', long = "stderr", conflicts_with = "output")]
    stderr: bool,

    /// Command and arguments to run
    #[arg(last = true, required = true)]
    command: Vec<String>,
}

enum Output {
    Stdout(BufWriter<tokio::io::Stdout>),
    Stderr(BufWriter<tokio::io::Stderr>),
    File(BufWriter<tokio::fs::File>),
}

impl Output {
    async fn write_line(&mut self, prefix: &str, line: &str) -> Result<()> {
        let buf = format!("{} {}", prefix, line);
        match self {
            Output::Stdout(w) => w.write_all(buf.as_bytes()).await?,
            Output::Stderr(w) => w.write_all(buf.as_bytes()).await?,
            Output::File(w) => w.write_all(buf.as_bytes()).await?,
        }
        self.flush().await?;
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        match self {
            Output::Stdout(w) => w.flush().await?,
            Output::Stderr(w) => w.flush().await?,
            Output::File(w) => w.flush().await?,
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let mut cmd = Command::new(&args.command[0]);
    cmd.args(&args.command[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn child process")?;
    
    let mut output = if args.stderr {
        Output::Stderr(BufWriter::new(tokio::io::stderr()))
    } else if let Some(path) = args.output {
        Output::File(BufWriter::new(tokio::fs::File::create(path).await?))
    } else {
        Output::Stdout(BufWriter::new(tokio::io::stdout()))
    };

    let mut stdin_reader = BufReader::new(tokio::io::stdin());
    let mut stdin_line = String::new();
    
    let mut stdout = BufReader::new(child.stdout.take()
        .context("Failed to get child stdout")?);
    let mut stdout_line = String::new();
    
    let mut stderr = BufReader::new(child.stderr.take()
        .context("Failed to get child stderr")?);
    let mut stderr_line = String::new();
    
    let mut child_stdin = Some(child.stdin.take()
        .context("Failed to get child stdin")?);

    let mut stdin_closed = false;

    loop {
        select! {
            result = stdin_reader.read_line(&mut stdin_line), if !stdin_closed => {
                match result {
                    Ok(0) => {
                        stdin_closed = true;
                        drop(child_stdin.take());
                    }
                    Ok(_) => {
                        if let Some(ref mut stdin) = child_stdin {
                            stdin.write_all(stdin_line.as_bytes()).await?;
                            output.write_line("-->", &stdin_line).await?;
                            stdin_line.clear();
                        }
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            
            result = stdout.read_line(&mut stdout_line) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        tokio::io::stdout().write_all(stdout_line.as_bytes()).await?;
                        output.write_line("<--", &stdout_line).await?;
                        stdout_line.clear();
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            
            result = stderr.read_line(&mut stderr_line) => {
                match result {
                    Ok(0) => break,
                    Ok(_) => {
                        tokio::io::stderr().write_all(stderr_line.as_bytes()).await?;
                        output.write_line("!--", &stderr_line).await?;
                        stderr_line.clear();
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            
            result = child.wait(), if stdin_closed => {
                match result {
                    Ok(_) => break,
                    Err(e) => return Err(e.into()),
                }
            }
        }
    }

    Ok(())
}
