use structopt::StructOpt;
use structopt::clap::arg_enum;
use serde_json::{json};

arg_enum! {
    #[derive(Debug)]
    enum OutputFormat {
      Table, JSON, TOML, YAML,
    }
}

#[derive(StructOpt, Debug)]
struct Opt {
    /// Concurrency level, how many calls are made in parallel [default: 1]
    #[structopt(long, short)]
    concurrency: Option<usize>,

    /// How long to run [default: 10]
    #[structopt(long, short)]
    duration: Option<usize>,

    /// Time out to wait for a response before aborting the call [default: 60]
    #[structopt(long, short)]
    time_out: Option<usize>,

    /// How many tasks to spawn per concurrent connection [default: 10]
    /// the ratio will be increased by 10% everytime the task queue gets empty
    #[structopt(long, short)]
    ratio: Option<usize>,

    /// Output format [default: table]
    //#[structopt(long, short, default_value="table")]
    #[structopt(long, short, possible_values=&OutputFormat::variants(), case_insensitive=true)]
    format: Option<OutputFormat>,

    /// URL or Settings file, 
    /// if none is passed a pickaxe.yml will be looked for in the current path
    /// flags take precedence over file settings
    #[structopt(parse(from_str), default_value="pickaxe.yml")]
    target: String,

}

fn print_table(result: &pickaxe::result::Result) {
  let half_row_lenght = 15;
  let left = half_row_lenght;
  let right = match &result.url {
    Some(url) => {
      if half_row_lenght < url.path().len() {
        url.path().len()
      } else { half_row_lenght}
    },
    _ => half_row_lenght,
  };
  let width = left + right;

  if let Some(url) = &result.url {
    println!("{:^width$}", "Benchmarking", width = width);
    println!("{:=<width$}", "=", width = width);
    println!("{:<left$}{:>right$}", "Schema:", url.scheme(),  left = left, right = right);
    if let Some(host) = url.host_str() {
      println!("{:<left$}{:>right$}", "Host:", host,  left = left, right = right);
    }
    if let Some(port) = url.port() {
      println!("{:<left$}{:>right$}", "Port:", port,  left = left, right = right);
    }
    println!("{:<left$}{:>right$}", "Path:", url.path(),  left = left, right = right);
  }
  println!("{:<left$}{:>right$}", "Concurrency:", result.concurrency_level,  left = left, right = right);
  println!("{:<left$}{:>right$}", "Task ratio:", result.ratio / result.concurrency_level,  left = left, right = right);
  println!("");
  println!("{:^width$}", "Requests", width = width);
  println!("{:=<width$}", "=", width = width);
  println!("{:<left$}{:>right$}", "Completed:", result.total(), left = left, right = right);
  println!("{:<left$}{:>right$}", "Duration:", format!("{:?}", result.duration),  left = left, right = right);
  println!("{:<left$}{:>right$}", "Success:", result.success, left = left, right = right);
  println!("{:<left$}{:>right$}", "Failed:", result.failed, left = left, right = right);
  println!("{:<left$}{:>right$}", "Timeout:", result.timed_out, left = left, right = right);
  println!("");
  println!("{:^width$}", "Response Status Code", width = width);
  println!("{:=<width$}", "=", width = width);
  for (k, v) in result.status_code_summary.iter() {
    println!("{:<left$}{:>right$}", format!("{:?}", k), v, left = left, right = right);
  }
  println!("");
  println!("{:^width$}", "Response time summary", width = width);
  println!("{:=<width$}", "=", width = width);
  for (k, v) in result.duration_summary.iter() {
    let perc = ((*v as f32 / result.total() as f32) * 100f32) as usize;
    print!("{:<left$}{:>right$}", format!("{:?}", k), v, left = left, right = right);
    println!(" {:+<width$}", "", width = perc );
  }
}

fn print_json(result: &pickaxe::result::Result) {
  let mut sc : Vec<(String, usize)> = Vec::new();
  for (code, qty) in &result.status_code_summary {
    sc.push((format!("{:?}", code), *qty));
  };
  let mut d : Vec<(String, usize)> = Vec::new();
  for (range, qty) in &result.duration_summary {
    d.push((format!("{:?}", range), *qty));
  };

  let res = json!({
    "url": result.url_str(),
    "concurrency_level": result.concurrency_level,
    "task_ratio": result.ratio,
    "requests": result.total(),
    "duration": result.duration,
    "success": result.success,
    "failed": result.failed,
    "timed_out": result.timed_out,
    "status_code": sc,
    "response_times": d,
  });
  println!("{}", res.to_string());
}



fn main() {
  let opt = Opt::from_args();
  let mut job = match pickaxe::job::new(opt.target) {
    Ok(job) => job,
    Err(e) => {
      println!("{}", e);
      panic!("aborting");
    },
  };
  if let Some(v) = opt.concurrency {
    job.concurrency = v;
  }

  if let Some(v) = opt.duration {
    job.duration = v;
  }

  if let Some(v) = opt.time_out {
    job.time_out = v;
  }

  let mut broker = pickaxe::broker::new(job);
  broker.run();

  match opt.format {
    Some(OutputFormat::JSON) => print_json(broker.get_result()),
    _ => print_table(broker.get_result()),
  }
}
