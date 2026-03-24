use clap::{ArgAction, Args, Parser};

#[derive(Debug, Clone, Parser)]
#[command(name = "rsenv", version, about = "检测并打印机器环境信息。")]
pub struct Cli {
    #[command(flatten)]
    pub categories: CategoryArgs,

    #[command(flatten)]
    pub output: OutputArgs,

    #[arg(long, help = "采集所有支持的机器环境分类")]
    pub all: bool,

    #[arg(long, value_name = "NAME", help = "仅采集一个 helper 条目并输出 JSON")]
    pub helper: Option<String>,
}

#[derive(Debug, Clone, Args, Default)]
pub struct CategoryArgs {
    #[arg(long, help = "输出系统信息")]
    pub system: bool,

    #[arg(long, help = "输出浏览器版本")]
    pub browser: bool,

    #[arg(long, help = "输出 SDK 信息")]
    pub sdk: bool,

    #[arg(long, help = "输出 IDE 信息")]
    pub ide: bool,

    #[arg(long, help = "输出语言版本")]
    pub languages: bool,

    #[arg(long, help = "输出包管理器版本")]
    pub manager: bool,

    #[arg(long, help = "输出二进制工具版本")]
    pub binary: bool,

    #[arg(long, help = "输出服务端软件版本")]
    pub server: bool,

    #[arg(long, help = "输出虚拟化工具信息")]
    pub r#virtual: bool,

    #[arg(long, help = "输出常用工具版本")]
    pub util: bool,

    #[arg(long, help = "输出数据库版本")]
    pub database: bool,
}

#[derive(Debug, Clone, Args)]
pub struct OutputArgs {
    #[arg(
        long,
        conflicts_with_all = ["markdown", "toml"],
        help = "以 JSON 格式输出"
    )]
    pub json: bool,

    #[arg(
        long,
        conflicts_with_all = ["json", "toml"],
        help = "以 Markdown 格式输出"
    )]
    pub markdown: bool,

    #[arg(
        long,
        conflicts_with_all = ["json", "markdown"],
        help = "以 TOML 格式输出"
    )]
    pub toml: bool,

    #[arg(long = "showNotFound", visible_alias = "show-not-found", action = ArgAction::SetTrue, help = "包含 Not Found 条目")]
    pub show_not_found: bool,

    #[arg(long, help = "包含重复版本信息")]
    pub duplicates: bool,

    #[arg(
        long = "fullTree",
        visible_alias = "full-tree",
        help = "显示完整依赖树"
    )]
    pub full_tree: bool,
}

pub fn parse_args() -> Cli {
    Cli::parse()
}
