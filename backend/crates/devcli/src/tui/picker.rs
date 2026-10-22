use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerOutcome {
    Open,
    Chosen(String),
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picker {
    pub title: String,
    pub items: Vec<String>,
    pub selected: usize,
}

impl Picker {
    pub fn new(title: &str, items: Vec<String>, current: Option<&str>) -> Self {
        let selected = current
            .and_then(|current| items.iter().position(|item| item == current))
            .unwrap_or(0);
        Self {
            title: title.to_owned(),
            items,
            selected,
        }
    }

    pub fn handle(&mut self, key: KeyEvent) -> PickerOutcome {
        let last = self.items.len().saturating_sub(1);
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.selected = (self.selected + 1).min(last),
            KeyCode::Up | KeyCode::Char('k') => self.selected = self.selected.saturating_sub(1),
            KeyCode::Enter => {
                return self
                    .items
                    .get(self.selected)
                    .map_or(PickerOutcome::Cancelled, |item| {
                        PickerOutcome::Chosen(item.clone())
                    });
            }
            KeyCode::Esc | KeyCode::Char('q') => return PickerOutcome::Cancelled,
            _ => {}
        }
        PickerOutcome::Open
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    fn press(picker: &mut Picker, code: KeyCode) -> PickerOutcome {
        picker.handle(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn starts_on_the_current_item_and_chooses_with_enter() {
        let mut picker = Picker::new(
            "Profile",
            vec!["full".into(), "minimal".into(), "prod-like".into()],
            Some("minimal"),
        );
        assert_eq!(picker.selected, 1);
        press(&mut picker, KeyCode::Char('j'));
        press(&mut picker, KeyCode::Char('j'));
        assert_eq!(picker.selected, 2);
        assert_eq!(
            press(&mut picker, KeyCode::Enter),
            PickerOutcome::Chosen("prod-like".into())
        );
        press(&mut picker, KeyCode::Up);
        assert_eq!(picker.selected, 1);
        assert_eq!(press(&mut picker, KeyCode::Esc), PickerOutcome::Cancelled);
        assert_eq!(press(&mut picker, KeyCode::Char('x')), PickerOutcome::Open);
    }

    #[test]
    fn an_empty_picker_cannot_choose() {
        let mut picker = Picker::new("Profile", Vec::new(), None);
        assert_eq!(press(&mut picker, KeyCode::Enter), PickerOutcome::Cancelled);
    }
}
