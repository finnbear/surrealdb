use super::run;
use crate::fnc::script::modules::impl_module_def;
use crate::sql::value::Value;
use js::{Created, Ctx, Func, Loaded, Module, ModuleDef, Native, Object, Rest, Result};

pub struct Package;

type Any = Rest<Value>;

impl_module_def!(
	Package,
	"is",
	"alphanum" => run,
	"alpha" => run,
	"ascii" => run,
	"datetime" => run,
	"domain" => run,
	"email" => run,
	"email" => run,
	"hexadecimal" => run,
	"latitude" => run,
	"longitude" => run,
	"numeric" => run,
	"semver" => run,
	"url" => run,
	"uuid" => run
);
