//entry point , start app

//declaring module files
mod app ;
mod screen ;
mod input ;
mod life ;
mod render ;

use app::App ;

fn main() -> std::io::Result<()>
{
    let mut app = App::new() ;  //making new , alloc mem
    app.run()
}
