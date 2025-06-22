use super::manager::ScreenManager;
use super::screen::Writer;

pub trait ScreenManagerInterface {
    fn with_writer<F, R>(&mut self, screen_id: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut Writer) -> R;
}

impl ScreenManagerInterface for ScreenManager {
    fn with_writer<F, R>(&mut self, screen_id: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut Writer) -> R,
    {
        let screen = self.screens.get_mut(screen_id)?.as_mut()?;
        let mut writer = Writer::new(screen);
        Some(f(&mut writer))
    }
} 