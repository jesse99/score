extern crate rsimbase;

use rsimbase::*;

fn execute(env: &rsimbase::Env, context: &mut Context)
{
	let _ = env;
	context.log_info("executing");
}

fn main()
{
	rsimbase::run(execute);
}