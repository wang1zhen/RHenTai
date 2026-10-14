use std::path::PathBuf;
use std::sync::Arc;

use clap::{Args, Parser, Subcommand};
use rhentai_core::{AppPaths, Core, IpcError, NoopEventSink};

mod client_cmds;
mod core_cmds;

#[derive(Parser, Debug)]
#[command(name = "rhentai-cli", about = "RHenTai dev/QA command line tool")]
pub struct Cli {
    #[arg(long, global = true)]
    pub data_dir: Option<PathBuf>,

    #[arg(long, global = true, value_parser = ["eh", "ex"])]
    pub site: Option<String>,

    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check authentication status
    Status,

    /// Login using existing cookies
    LoginCookies {
        #[arg(long)]
        member_id: String,
        #[arg(long)]
        pass_hash: String,
        #[arg(long)]
        igneous: Option<String>,
    },

    /// Login using username and password
    LoginPassword {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },

    /// Log out and clear cookies
    Logout,

    /// List galleries
    List {
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        exclude_cats: Option<String>,
        #[arg(long)]
        cursor: Option<String>,
    },

    /// Show gallery details
    Detail { gallery: String },

    /// Show thumbnail page
    Thumbs {
        gallery: String,
        #[arg(long)]
        page: u32,
    },

    /// Resolve and download one page image
    Image {
        gallery: String,
        #[arg(long)]
        index: u32,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        reload: bool,
    },

    /// List favorite categories
    Favcats,

    /// Add, update, or remove a favorite
    FavSet {
        gallery: String,
        #[arg(long)]
        slot: Option<u8>,
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        remove: bool,
    },

    /// Download galleries
    Download {
        #[arg(required = true, num_args = 1..)]
        galleries: Vec<String>,
        #[arg(long)]
        original: bool,
        #[arg(long)]
        concurrency: Option<u8>,
        #[arg(long)]
        dir: Option<PathBuf>,
    },

    /// List download tasks
    Downloads,

    /// Local library operations
    Library(LibraryArgs),
}

#[derive(Args, Debug)]
pub struct LibraryArgs {
    #[command(subcommand)]
    pub command: LibraryCommands,
}

#[derive(Subcommand, Debug)]
pub enum LibraryCommands {
    /// Scan directories into the library
    Scan {
        #[arg(long = "dir")]
        dirs: Vec<PathBuf>,
    },
    /// List pages of a local gallery
    Pages { id: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let mut paths = AppPaths::from_dirs();
    if let Some(ref data_dir) = cli.data_dir {
        paths = paths.with_data_dir_override(data_dir);
    }

    let core = match Core::open(paths, Arc::new(NoopEventSink)) {
        Ok(c) => c,
        Err(e) => {
            handle_error(&e);
            return;
        }
    };

    if let Some(ref site_str) = cli.site {
        let site = match site_str.as_str() {
            "ex" => eh_client::Site::Ex,
            _ => eh_client::Site::Eh,
        };
        let mut s = core.settings.get();
        s.site = site;
        let _ = core.settings.update(s);
    }

    let json = cli.json;
    let res = match cli.command {
        Commands::Status => client_cmds::status(&core, json).await,
        Commands::LoginCookies {
            member_id,
            pass_hash,
            igneous,
        } => client_cmds::login_cookies(&core, member_id, pass_hash, igneous, json).await,
        Commands::LoginPassword { username, password } => {
            client_cmds::login_password(&core, username, password, json).await
        }
        Commands::Logout => client_cmds::logout(&core, json).await,
        Commands::List {
            source,
            query,
            exclude_cats,
            cursor,
        } => client_cmds::list(&core, source, query, exclude_cats, cursor, json).await,
        Commands::Detail { gallery } => client_cmds::detail(&core, gallery, json).await,
        Commands::Thumbs { gallery, page } => client_cmds::thumbs(&core, gallery, page, json).await,
        Commands::Image {
            gallery,
            index,
            out,
            reload,
        } => client_cmds::image(&core, gallery, index, out, reload, json).await,
        Commands::Favcats => client_cmds::favcats(&core, json).await,
        Commands::FavSet {
            gallery,
            slot,
            note,
            remove,
        } => client_cmds::fav_set(&core, gallery, slot, note, remove, json).await,
        Commands::Download {
            galleries,
            original,
            concurrency,
            dir,
        } => core_cmds::download(&core, galleries, original, concurrency, dir, json).await,
        Commands::Downloads => core_cmds::downloads(&core, json).await,
        Commands::Library(lib_args) => match lib_args.command {
            LibraryCommands::Scan { dirs } => core_cmds::library_scan(&core, dirs, json).await,
            LibraryCommands::Pages { id } => core_cmds::library_pages(&core, id, json).await,
        },
    };

    if let Err(e) = res {
        handle_error(&e);
    }
}

fn handle_error(err: &IpcError) {
    eprintln!("error[{:?}]: {}", err.kind, err.message);
    std::process::exit(1);
}
