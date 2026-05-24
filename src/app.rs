// state machine , orchastrator

pub struct App
{
    mode: Mode ,
}

impl App{

    pub fn new() -> Self
    {
        Self
        {
            mode: Mode::Normal,
        }
    }

    pub fn run( &mut self ) -> std::io::Result<()> //can modify feilds since mut
    {
        //println! ("its TIDEing time..");
        loop
        {
            match self.mode //switch
            {
                Mode::Normal=>
                {
                    break ;
                }

                Mode::Screensaver=>
                {
                    break ;
                }
            }
        }
        Ok( () )
    }

}


pub enum Mode {
    Normal,
    Screensaver,
}