use std::collections::{BTreeSet, HashSet};

use crate::app::shell::ShellType;
use crate::catalog::types::{Alias, AliasCatalog};
use crate::core::validation::{is_valid_alias_name, is_valid_tag};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorMode {
    Single,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Search,
    List,
    Name,
    Command,
    Description,
    Tags,
    Enabled,
    Global,
    Add,
    Delete,
    Save,
    Cancel,
}

impl Focus {
    pub fn is_text(self) -> bool {
        matches!(
            self,
            Self::Search | Self::Name | Self::Command | Self::Description | Self::Tags
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputField {
    pub value: String,
    pub cursor: usize,
}

impl InputField {
    pub fn new(value: String) -> Self {
        let cursor = value.chars().count();
        Self { value, cursor }
    }
    pub fn insert(&mut self, character: char) {
        let byte = byte_index(&self.value, self.cursor);
        self.value.insert(byte, character);
        self.cursor += 1;
    }
    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let start = byte_index(&self.value, self.cursor - 1);
        let end = byte_index(&self.value, self.cursor);
        self.value.replace_range(start..end, "");
        self.cursor -= 1;
    }
    pub fn delete(&mut self) {
        if self.cursor == self.value.chars().count() {
            return;
        }
        let start = byte_index(&self.value, self.cursor);
        let end = byte_index(&self.value, self.cursor + 1);
        self.value.replace_range(start..end, "");
    }
    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }
    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.value.chars().count());
    }
}

fn byte_index(value: &str, character_index: usize) -> usize {
    value
        .char_indices()
        .nth(character_index)
        .map_or(value.len(), |(index, _)| index)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AliasDraft {
    pub id: u64,
    pub original_name: Option<String>,
    pub name: InputField,
    pub command: InputField,
    pub description: InputField,
    pub description_present: bool,
    pub description_touched: bool,
    pub tags: InputField,
    pub enabled: bool,
    pub global: bool,
}

impl AliasDraft {
    fn from_alias(id: u64, name: &str, alias: &Alias) -> Self {
        Self {
            id,
            original_name: Some(name.to_owned()),
            name: InputField::new(name.to_owned()),
            command: InputField::new(alias.command.clone()),
            description: InputField::new(alias.description.clone().unwrap_or_default()),
            description_present: alias.description.is_some(),
            description_touched: false,
            tags: InputField::new(alias.tags.iter().cloned().collect::<Vec<_>>().join(", ")),
            enabled: alias.enabled,
            global: alias.global,
        }
    }
    fn empty(id: u64) -> Self {
        Self {
            id,
            original_name: None,
            name: InputField::new(String::new()),
            command: InputField::new(String::new()),
            description: InputField::new(String::new()),
            description_present: false,
            description_touched: false,
            tags: InputField::new(String::new()),
            enabled: true,
            global: false,
        }
    }
    pub fn parse(&self) -> Result<(String, Alias), Vec<String>> {
        let mut errors = Vec::new();
        if !is_valid_alias_name(&self.name.value) {
            errors.push("Alias names cannot be empty or contain whitespace or '='.".to_owned());
        }
        if self.command.value.trim().is_empty() {
            errors.push("Commands cannot be empty.".to_owned());
        }
        let tags = match parse_tags(&self.tags.value) {
            Ok(tags) => tags,
            Err(error) => {
                errors.push(error);
                BTreeSet::new()
            }
        };
        if !errors.is_empty() {
            return Err(errors);
        }
        let description = if self.description.value.is_empty() {
            (self.description_present && !self.description_touched).then(String::new)
        } else {
            Some(self.description.value.clone())
        };
        Ok((
            self.name.value.clone(),
            Alias {
                command: self.command.value.clone(),
                enabled: self.enabled,
                global: self.global,
                description,
                tags,
                detailed: false,
            },
        ))
    }
    fn searchable_text(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.name.value, self.command.value, self.description.value, self.tags.value
        )
        .to_lowercase()
    }
}

pub fn parse_tags(value: &str) -> Result<BTreeSet<String>, String> {
    let tags = value
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if let Some(tag) = tags.iter().find(|tag| !is_valid_tag(tag)) {
        return Err(format!(
            "Tag '{tag}' is invalid; tags cannot contain whitespace."
        ));
    }
    Ok(tags)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Modal {
    Replace {
        source_id: u64,
        target_id: Option<u64>,
        name: String,
        choice: usize,
    },
    Delete {
        id: u64,
        name: String,
        choice: usize,
    },
    DirtyExit {
        choice: usize,
    },
    Help,
}

impl Modal {
    #[cfg(test)]
    pub fn choice(&self) -> Option<usize> {
        match self {
            Self::Replace { choice, .. }
            | Self::Delete { choice, .. }
            | Self::DirtyExit { choice } => Some(*choice),
            Self::Help => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum EditorResult {
    Continue,
    ExitNoChanges,
    Save(AliasCatalog),
}

pub struct EditorState {
    pub mode: EditorMode,
    pub shell: ShellType,
    pub original: AliasCatalog,
    pub drafts: Vec<AliasDraft>,
    pub selected_id: Option<u64>,
    pub search: InputField,
    pub focus: Focus,
    pub modal: Option<Modal>,
    pub status: Option<String>,
    pub compact_form: bool,
    pub compact_layout: bool,
    next_id: u64,
    confirmed_external_replacements: HashSet<String>,
    last_edited_id: Option<u64>,
}

impl EditorState {
    pub fn single(catalog: &AliasCatalog, name: &str, shell: ShellType) -> Result<Self, String> {
        let alias = catalog
            .aliases
            .get(name)
            .ok_or_else(|| format!("Alias '{name}' does not exist."))?;
        let draft = AliasDraft::from_alias(0, name, alias);
        Ok(Self {
            mode: EditorMode::Single,
            shell,
            original: catalog.clone(),
            drafts: vec![draft],
            selected_id: Some(0),
            search: InputField::new(String::new()),
            focus: Focus::Name,
            modal: None,
            status: None,
            compact_form: true,
            compact_layout: false,
            next_id: 1,
            confirmed_external_replacements: HashSet::new(),
            last_edited_id: Some(0),
        })
    }
    pub fn all(catalog: &AliasCatalog, shell: ShellType) -> Self {
        let drafts = catalog
            .aliases
            .iter()
            .enumerate()
            .map(|(id, (name, alias))| AliasDraft::from_alias(id as u64, name, alias))
            .collect::<Vec<_>>();
        let selected_id = drafts.first().map(|draft| draft.id);
        Self {
            mode: EditorMode::All,
            shell,
            original: catalog.clone(),
            next_id: drafts.len() as u64,
            drafts,
            selected_id,
            search: InputField::new(String::new()),
            focus: Focus::Search,
            modal: None,
            status: None,
            compact_form: false,
            compact_layout: false,
            confirmed_external_replacements: HashSet::new(),
            last_edited_id: selected_id,
        }
    }
    pub fn selected(&self) -> Option<&AliasDraft> {
        let id = self.selected_id?;
        self.drafts.iter().find(|draft| draft.id == id)
    }
    pub fn selected_mut(&mut self) -> Option<&mut AliasDraft> {
        let id = self.selected_id?;
        self.drafts.iter_mut().find(|draft| draft.id == id)
    }
    pub fn filtered_ids(&self) -> Vec<u64> {
        let query = self.search.value.to_lowercase();
        self.drafts
            .iter()
            .filter(|draft| query.is_empty() || draft.searchable_text().contains(&query))
            .map(|draft| draft.id)
            .collect()
    }
    pub fn validation_errors(&self) -> Vec<String> {
        self.drafts
            .iter()
            .flat_map(|draft| {
                draft
                    .parse()
                    .err()
                    .unwrap_or_default()
                    .into_iter()
                    .map(move |error| {
                        let name = if draft.name.value.is_empty() {
                            "new alias"
                        } else {
                            &draft.name.value
                        };
                        format!("{name}: {error}")
                    })
            })
            .collect()
    }
    pub fn is_dirty(&self) -> bool {
        self.preview_catalog(false).map_or(true, |catalog| {
            !semantic_catalog_eq(&catalog, &self.original)
        })
    }
    pub fn add_alias(&mut self) {
        let id = self.next_id;
        self.next_id += 1;
        self.drafts.push(AliasDraft::empty(id));
        self.selected_id = Some(id);
        self.last_edited_id = Some(id);
        self.focus = Focus::Name;
        self.compact_form = true;
        self.status = Some("New alias added to the draft.".to_owned());
    }
    pub fn request_delete(&mut self) {
        let Some(draft) = self.selected() else {
            self.status = Some("No alias is selected.".to_owned());
            return;
        };
        self.modal = Some(Modal::Delete {
            id: draft.id,
            name: draft.name.value.clone(),
            choice: 1,
        });
    }
    pub fn request_quit(&mut self) -> EditorResult {
        if self.is_dirty() {
            self.modal = Some(Modal::DirtyExit { choice: 2 });
            EditorResult::Continue
        } else {
            EditorResult::ExitNoChanges
        }
    }
    pub fn request_save(&mut self) -> EditorResult {
        self.status = None;
        let errors = self.validation_errors();
        if !errors.is_empty() {
            self.status = Some(errors.join(" "));
            return EditorResult::Continue;
        }
        if let Some((source_id, target_id, name)) = self.find_collision() {
            self.modal = Some(Modal::Replace {
                source_id,
                target_id,
                name,
                choice: 1,
            });
            return EditorResult::Continue;
        }
        match self.preview_catalog(true) {
            Ok(catalog) if semantic_catalog_eq(&catalog, &self.original) => {
                EditorResult::ExitNoChanges
            }
            Ok(catalog) => EditorResult::Save(catalog),
            Err(error) => {
                self.status = Some(error);
                EditorResult::Continue
            }
        }
    }
    fn find_collision(&self) -> Option<(u64, Option<u64>, String)> {
        let source_id = self.last_edited_id.or(self.selected_id)?;
        let source = self.drafts.iter().find(|draft| draft.id == source_id)?;
        let name = source.name.value.clone();
        if let Some(target) = self
            .drafts
            .iter()
            .find(|draft| draft.id != source_id && draft.name.value == name)
        {
            return Some((source_id, Some(target.id), name));
        }
        if self.mode == EditorMode::Single
            && source.original_name.as_deref() != Some(name.as_str())
            && self.original.aliases.contains_key(&name)
            && !self.confirmed_external_replacements.contains(&name)
        {
            return Some((source_id, None, name));
        }
        for (index, source) in self.drafts.iter().enumerate() {
            if let Some(target) = self.drafts[..index]
                .iter()
                .find(|target| target.name.value == source.name.value)
            {
                return Some((source.id, Some(target.id), source.name.value.clone()));
            }
        }
        None
    }
    fn preview_catalog(&self, _require_valid: bool) -> Result<AliasCatalog, String> {
        let mut catalog = match self.mode {
            EditorMode::Single => self.original.clone(),
            EditorMode::All => AliasCatalog::new(),
        };
        if self.mode == EditorMode::Single
            && let Some(original_name) = self
                .drafts
                .first()
                .and_then(|draft| draft.original_name.as_ref())
        {
            catalog.aliases.remove(original_name);
        }
        for draft in &self.drafts {
            let (name, alias) = draft.parse().map_err(|errors| errors.join(" "))?;
            if self.confirmed_external_replacements.contains(&name) {
                catalog.aliases.remove(&name);
            }
            catalog.aliases.insert(name, alias);
        }
        Ok(catalog)
    }
    pub fn move_selection(&mut self, delta: isize) {
        let ids = self.filtered_ids();
        if ids.is_empty() {
            self.selected_id = None;
            return;
        }
        let current = self
            .selected_id
            .and_then(|selected| ids.iter().position(|id| *id == selected))
            .unwrap_or(0) as isize;
        self.selected_id = Some(ids[(current + delta).clamp(0, ids.len() as isize - 1) as usize]);
    }
    pub fn select(&mut self, id: u64) {
        if self.drafts.iter().any(|draft| draft.id == id) {
            self.selected_id = Some(id);
        }
    }
    pub fn cycle_focus(&mut self, backwards: bool) {
        let focuses = self.focus_order();
        let index = focuses
            .iter()
            .position(|focus| *focus == self.focus)
            .unwrap_or(0);
        let next = if backwards {
            (index + focuses.len() - 1) % focuses.len()
        } else {
            (index + 1) % focuses.len()
        };
        self.focus = focuses[next];
    }
    fn focus_order(&self) -> Vec<Focus> {
        if self.mode == EditorMode::All && self.compact_layout && !self.compact_form {
            return vec![
                Focus::Search,
                Focus::List,
                Focus::Add,
                Focus::Delete,
                Focus::Save,
                Focus::Cancel,
            ];
        }
        let mut fields = Vec::new();
        if self.mode == EditorMode::All && !self.compact_layout {
            fields.extend([Focus::Search, Focus::List]);
        }
        fields.extend([
            Focus::Name,
            Focus::Command,
            Focus::Description,
            Focus::Tags,
            Focus::Enabled,
        ]);
        if self.shell == ShellType::Zsh {
            fields.push(Focus::Global);
        }
        if self.mode == EditorMode::All && !self.compact_layout {
            fields.extend([Focus::Add, Focus::Delete]);
        }
        fields.extend([Focus::Save, Focus::Cancel]);
        fields
    }
    pub fn toggle_focused(&mut self) {
        self.status = None;
        let focus = self.focus;
        let shell = self.shell.clone();
        let Some(draft) = self.selected_mut() else {
            return;
        };
        match focus {
            Focus::Enabled => draft.enabled = !draft.enabled,
            Focus::Global if shell == ShellType::Zsh => draft.global = !draft.global,
            _ => return,
        }
        let id = draft.id;
        self.last_edited_id = Some(id);
    }
    pub fn edit_focused(&mut self, action: TextAction) {
        self.status = None;
        let focus = self.focus;
        let field = if focus == Focus::Search {
            Some(&mut self.search)
        } else {
            self.selected_mut().and_then(|draft| match focus {
                Focus::Name => Some(&mut draft.name),
                Focus::Command => Some(&mut draft.command),
                Focus::Description => {
                    draft.description_touched = true;
                    Some(&mut draft.description)
                }
                Focus::Tags => Some(&mut draft.tags),
                _ => None,
            })
        };
        let Some(field) = field else {
            return;
        };
        match action {
            TextAction::Insert(character) => field.insert(character),
            TextAction::Backspace => field.backspace(),
            TextAction::Delete => field.delete(),
            TextAction::Left => field.move_left(),
            TextAction::Right => field.move_right(),
            TextAction::Home => field.cursor = 0,
            TextAction::End => field.cursor = field.value.chars().count(),
        }
        if focus != Focus::Search {
            self.last_edited_id = self.selected_id;
            self.confirmed_external_replacements.clear();
        } else {
            self.selected_id = self.filtered_ids().first().copied();
        }
    }
    pub fn move_modal_choice(&mut self, delta: isize) {
        let Some(modal) = self.modal.as_mut() else {
            return;
        };
        let (choice, count) = match modal {
            Modal::Replace { choice, .. } | Modal::Delete { choice, .. } => (choice, 2),
            Modal::DirtyExit { choice } => (choice, 3),
            Modal::Help => return,
        };
        *choice = ((*choice as isize + delta).rem_euclid(count)) as usize;
    }
    pub fn set_modal_choice(&mut self, selected: usize) {
        let Some(modal) = self.modal.as_mut() else {
            return;
        };
        match modal {
            Modal::Replace { choice, .. } | Modal::Delete { choice, .. } => {
                *choice = selected.min(1)
            }
            Modal::DirtyExit { choice } => *choice = selected.min(2),
            Modal::Help => {}
        }
    }
    pub fn dismiss_modal(&mut self) {
        self.modal = None;
    }
    pub fn activate_modal(&mut self) -> EditorResult {
        let Some(modal) = self.modal.take() else {
            return EditorResult::Continue;
        };
        match modal {
            Modal::Replace {
                source_id,
                target_id,
                name,
                choice,
            } => {
                if choice == 0 {
                    if let Some(target_id) = target_id {
                        self.drafts.retain(|draft| draft.id != target_id);
                    } else {
                        self.confirmed_external_replacements.insert(name);
                    }
                    self.selected_id = Some(source_id);
                    self.request_save()
                } else {
                    self.status = Some("Replacement declined; changes were not saved.".to_owned());
                    EditorResult::Continue
                }
            }
            Modal::Delete { id, choice, .. } => {
                if choice == 0 {
                    self.drafts.retain(|draft| draft.id != id);
                    self.selected_id = self.filtered_ids().first().copied();
                    self.status = Some("Alias removed from the draft.".to_owned());
                }
                EditorResult::Continue
            }
            Modal::DirtyExit { choice } => match choice {
                0 => self.request_save(),
                1 => EditorResult::ExitNoChanges,
                _ => EditorResult::Continue,
            },
            Modal::Help => EditorResult::Continue,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAction {
    Insert(char),
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
}

fn semantic_catalog_eq(left: &AliasCatalog, right: &AliasCatalog) -> bool {
    left.aliases.len() == right.aliases.len()
        && left.aliases.iter().all(|(name, alias)| {
            right.aliases.get(name).is_some_and(|other| {
                alias.command == other.command
                    && alias.enabled == other.enabled
                    && alias.global == other.global
                    && alias.description == other.description
                    && alias.tags == other.tags
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn catalog() -> AliasCatalog {
        let mut catalog = AliasCatalog::new();
        let mut ll = Alias::new("ls -la".into(), true, false);
        ll.description = Some("List files".into());
        ll.tags.extend(["files".into(), "shell".into()]);
        catalog.aliases.insert("ll".into(), ll);
        catalog
            .aliases
            .insert("test".into(), Alias::new("cargo test".into(), true, false));
        catalog
    }
    #[test]
    fn tags_are_trimmed_sorted_and_deduplicated() {
        assert_eq!(
            parse_tags("rust, dev, rust")
                .unwrap()
                .into_iter()
                .collect::<Vec<_>>(),
            ["dev", "rust"]
        );
        assert!(parse_tags("two words").is_err());
    }
    #[test]
    fn single_edit_can_rename_and_replace_after_confirmation() {
        let catalog = catalog();
        let mut state = EditorState::single(&catalog, "ll", ShellType::Zsh).unwrap();
        state.selected_mut().unwrap().name = InputField::new("test".into());
        state.last_edited_id = state.selected_id;
        assert_eq!(state.request_save(), EditorResult::Continue);
        assert!(matches!(state.modal, Some(Modal::Replace { .. })));
        state.move_modal_choice(-1);
        let EditorResult::Save(saved) = state.activate_modal() else {
            panic!("replacement should save")
        };
        assert_eq!(saved.aliases.len(), 1);
        assert_eq!(saved.aliases["test"].command, "ls -la");
    }
    #[test]
    fn declined_replacement_stays_in_editor() {
        let catalog = catalog();
        let mut state = EditorState::single(&catalog, "ll", ShellType::Zsh).unwrap();
        state.selected_mut().unwrap().name = InputField::new("test".into());
        state.last_edited_id = state.selected_id;
        state.request_save();
        assert_eq!(state.activate_modal(), EditorResult::Continue);
        assert_eq!(state.selected().unwrap().name.value, "test");
        assert_eq!(state.original, catalog);
    }
    #[test]
    fn full_editor_searches_every_displayed_text_field() {
        let mut state = EditorState::all(&catalog(), ShellType::Bash);
        for query in ["ll", "ls -la", "list files", "shell"] {
            state.search = InputField::new(query.into());
            assert_eq!(state.filtered_ids().len(), 1, "{query}");
        }
    }
    #[test]
    fn new_alias_defaults_and_confirmed_delete_are_in_memory_only() {
        let catalog = catalog();
        let mut state = EditorState::all(&catalog, ShellType::Bash);
        state.add_alias();
        let new = state.selected().unwrap();
        assert!(new.enabled);
        assert!(!new.global);
        assert!(new.command.value.is_empty());
        let original_id = state.drafts[0].id;
        state.selected_id = Some(original_id);
        state.request_delete();
        state.move_modal_choice(-1);
        assert_eq!(state.activate_modal(), EditorResult::Continue);
        assert_eq!(state.drafts.len(), 2);
        assert_eq!(state.original, catalog);
    }
    #[test]
    fn dirty_exit_defaults_to_cancel_and_can_discard() {
        let mut state = EditorState::single(&catalog(), "ll", ShellType::Bash).unwrap();
        state.selected_mut().unwrap().command.insert('!');
        assert_eq!(state.request_quit(), EditorResult::Continue);
        assert_eq!(state.modal.as_ref().and_then(Modal::choice), Some(2));
        assert_eq!(state.activate_modal(), EditorResult::Continue);
        state.request_quit();
        state.move_modal_choice(-1);
        assert_eq!(state.activate_modal(), EditorResult::ExitNoChanges);
    }
    #[test]
    fn bash_focus_order_hides_global_but_preserves_its_value() {
        let mut catalog = catalog();
        catalog.aliases.get_mut("ll").unwrap().global = true;
        let mut state = EditorState::single(&catalog, "ll", ShellType::Bash).unwrap();
        for _ in 0..12 {
            assert_ne!(state.focus, Focus::Global);
            state.cycle_focus(false);
        }
        assert!(matches!(state.request_save(), EditorResult::ExitNoChanges));
    }
    #[test]
    fn invalid_name_command_and_tags_disable_saving() {
        let mut state = EditorState::single(&catalog(), "ll", ShellType::Zsh).unwrap();
        let draft = state.selected_mut().unwrap();
        draft.name.value = "bad name".into();
        draft.command.value.clear();
        draft.tags.value = "bad tag".into();
        assert_eq!(state.request_save(), EditorResult::Continue);
        let status = state.status.unwrap();
        assert!(status.contains("Alias names"));
        assert!(status.contains("Commands"));
        assert!(status.contains("Tag 'bad tag'"));
    }
    #[test]
    fn full_editor_add_collision_replaces_only_after_confirmation() {
        let mut state = EditorState::all(&catalog(), ShellType::Bash);
        state.add_alias();
        let draft = state.selected_mut().unwrap();
        draft.name = InputField::new("ll".into());
        draft.command = InputField::new("eza -la".into());
        assert_eq!(state.request_save(), EditorResult::Continue);
        assert!(matches!(state.modal, Some(Modal::Replace { .. })));
        state.move_modal_choice(-1);
        let EditorResult::Save(saved) = state.activate_modal() else {
            panic!("accepted replacement should save")
        };
        assert_eq!(saved.aliases["ll"].command, "eza -la");
        assert_eq!(saved.aliases.len(), 2);
    }
    #[test]
    fn representation_only_differences_are_semantic_noops() {
        let mut catalog = catalog();
        catalog.aliases.get_mut("test").unwrap().detailed = true;
        let mut state = EditorState::single(&catalog, "test", ShellType::Bash).unwrap();
        assert!(!state.is_dirty());
        assert!(matches!(state.request_save(), EditorResult::ExitNoChanges));
    }
    #[test]
    fn compact_focus_cycle_only_visits_visible_view_controls() {
        let mut state = EditorState::all(&catalog(), ShellType::Bash);
        state.compact_layout = true;
        state.compact_form = false;
        for _ in 0..6 {
            assert!(matches!(
                state.focus,
                Focus::Search
                    | Focus::List
                    | Focus::Add
                    | Focus::Delete
                    | Focus::Save
                    | Focus::Cancel
            ));
            state.cycle_focus(false);
        }
        state.compact_form = true;
        state.focus = Focus::Name;
        for _ in 0..7 {
            assert!(matches!(
                state.focus,
                Focus::Name
                    | Focus::Command
                    | Focus::Description
                    | Focus::Tags
                    | Focus::Enabled
                    | Focus::Save
                    | Focus::Cancel
            ));
            state.cycle_focus(false);
        }
    }
    #[test]
    fn unicode_input_cursor_edits_character_boundaries() {
        let mut input = InputField::new("a🦀".into());
        input.move_left();
        input.insert('é');
        assert_eq!(input.value, "aé🦀");
        input.backspace();
        assert_eq!(input.value, "a🦀");
        input.delete();
        assert_eq!(input.value, "a");
    }
}
