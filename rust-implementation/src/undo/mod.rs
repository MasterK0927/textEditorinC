use crate::core::{Result, UndoRedoSystem, MAX_HISTORY};
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct UndoRedoStack<T: Clone> {
    undo_stack: VecDeque<T>,
    redo_stack: VecDeque<T>,
    max_history: usize,
}

impl<T: Clone> UndoRedoStack<T> {
    pub fn new() -> Self {
        Self::with_capacity(MAX_HISTORY)
    }

    pub fn with_capacity(max_history: usize) -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(max_history),
            redo_stack: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    pub fn len(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.undo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    fn enforce_capacity(&mut self) {
        while self.undo_stack.len() > self.max_history {
            self.undo_stack.pop_front();
        }
        while self.redo_stack.len() > self.max_history {
            self.redo_stack.pop_front();
        }
    }
}

impl<T: Clone> Default for UndoRedoStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> UndoRedoSystem<T> for UndoRedoStack<T> {
    fn save_state(&mut self, state: T) {
        self.undo_stack.push_back(state);
        self.redo_stack.clear(); // Clear redo stack when new state is saved
        self.enforce_capacity();
    }

    fn undo(&mut self) -> Option<T> {
        if let Some(current_state) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(current_state.clone());
            self.enforce_capacity();
            self.undo_stack.back().cloned()
        } else {
            None
        }
    }

    fn redo(&mut self) -> Option<T> {
        if let Some(state) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(state.clone());
            self.enforce_capacity();
            Some(state)
        } else {
            None
        }
    }

    fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorAction {
    Insert { position: usize, character: char },
    Delete { position: usize, character: char },
    InsertText { position: usize, text: String },
    DeleteText { position: usize, text: String },
}

impl EditorAction {
    pub fn inverse(&self) -> Self {
        match self {
            EditorAction::Insert { position, character } => {
                EditorAction::Delete { position: *position, character: *character }
            }
            EditorAction::Delete { position, character } => {
                EditorAction::Insert { position: *position, character: *character }
            }
            EditorAction::InsertText { position, text } => {
                EditorAction::DeleteText { position: *position, text: text.clone() }
            }
            EditorAction::DeleteText { position, text } => {
                EditorAction::InsertText { position: *position, text: text.clone() }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionHistory {
    actions: UndoRedoStack<EditorAction>,
    group_actions: bool,
    current_group: Vec<EditorAction>,
}

impl ActionHistory {
    pub fn new() -> Self {
        Self {
            actions: UndoRedoStack::new(),
            group_actions: false,
            current_group: Vec::new(),
        }
    }

    pub fn with_capacity(max_history: usize) -> Self {
        Self {
            actions: UndoRedoStack::with_capacity(max_history),
            group_actions: false,
            current_group: Vec::new(),
        }
    }

    pub fn start_group(&mut self) {
        self.group_actions = true;
        self.current_group.clear();
    }

    pub fn end_group(&mut self) {
        if self.group_actions && !self.current_group.is_empty() {
            // Save the group as a single compound action
            // For simplicity, we'll save the last action of the group
            // In a real implementation, you might want to create a CompoundAction type
            if let Some(last_action) = self.current_group.last() {
                self.actions.save_state(last_action.clone());
            }
        }
        self.group_actions = false;
        self.current_group.clear();
    }

    pub fn record_action(&mut self, action: EditorAction) {
        if self.group_actions {
            self.current_group.push(action);
        } else {
            self.actions.save_state(action);
        }
    }

    pub fn undo_action(&mut self) -> Option<EditorAction> {
        self.actions.undo().map(|action| action.inverse())
    }

    pub fn redo_action(&mut self) -> Option<EditorAction> {
        self.actions.redo()
    }

    pub fn can_undo(&self) -> bool {
        self.actions.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.actions.can_redo()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
        self.current_group.clear();
        self.group_actions = false;
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.actions.undo_count(), self.actions.redo_count())
    }
}

impl Default for ActionHistory {
    fn default() -> Self {
        Self::new()
    }
}

pub struct TimestampedAction<T> {
    pub action: T,
    pub timestamp: std::time::Instant,
}

impl<T> TimestampedAction<T> {
    pub fn new(action: T) -> Self {
        Self {
            action,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn age(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
}

impl<T: Clone> Clone for TimestampedAction<T> {
    fn clone(&self) -> Self {
        Self {
            action: self.action.clone(),
            timestamp: self.timestamp,
        }
    }
}

pub struct TimestampedHistory<T: Clone> {
    history: UndoRedoStack<TimestampedAction<T>>,
    max_age: std::time::Duration,
}

impl<T: Clone> TimestampedHistory<T> {
    pub fn new(max_age: std::time::Duration) -> Self {
        Self {
            history: UndoRedoStack::new(),
            max_age,
        }
    }

    pub fn save_action(&mut self, action: T) {
        let timestamped = TimestampedAction::new(action);
        self.history.save_state(timestamped);
        self.cleanup_old_actions();
    }

    pub fn undo(&mut self) -> Option<T> {
        self.cleanup_old_actions();
        self.history.undo().map(|timestamped| timestamped.action)
    }

    pub fn redo(&mut self) -> Option<T> {
        self.cleanup_old_actions();
        self.history.redo().map(|timestamped| timestamped.action)
    }

    fn cleanup_old_actions(&mut self) {
        // Remove actions older than max_age from both stacks
        // This is a simplified implementation - in practice you might want
        // to be more careful about maintaining stack integrity
        while let Some(action) = self.history.undo_stack.back() {
            if action.age() > self.max_age {
                self.history.undo_stack.pop_back();
            } else {
                break;
            }
        }

        while let Some(action) = self.history.redo_stack.back() {
            if action.age() > self.max_age {
                self.history.redo_stack.pop_back();
            } else {
                break;
            }
        }
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn clear(&mut self) {
        self.history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo_stack() {
        let mut stack = UndoRedoStack::new();

        // Test empty stack
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert!(stack.is_empty());

        // Save states
        stack.save_state("state1".to_string());
        stack.save_state("state2".to_string());
        stack.save_state("state3".to_string());

        assert!(stack.can_undo());
        assert!(!stack.can_redo());
        assert_eq!(stack.len(), 3);

        // Test undo
        let state = stack.undo();
        assert_eq!(state, Some("state2".to_string()));
        assert!(stack.can_undo());
        assert!(stack.can_redo());

        // Test redo
        let state = stack.redo();
        assert_eq!(state, Some("state3".to_string()));
        assert!(stack.can_undo());
        assert!(!stack.can_redo());
    }

    #[test]
    fn test_action_history() {
        let mut history = ActionHistory::new();

        let action1 = EditorAction::Insert { position: 0, character: 'H' };
        let action2 = EditorAction::Insert { position: 1, character: 'i' };

        history.record_action(action1.clone());
        history.record_action(action2.clone());

        assert!(history.can_undo());

        // Test undo (should return inverse action)
        let undo_action = history.undo_action();
        assert_eq!(undo_action, Some(EditorAction::Delete { position: 1, character: 'i' }));

        assert!(history.can_redo());

        // Test redo
        let redo_action = history.redo_action();
        assert_eq!(redo_action, Some(action2));
    }

    #[test]
    fn test_action_grouping() {
        let mut history = ActionHistory::new();

        history.start_group();
        history.record_action(EditorAction::Insert { position: 0, character: 'H' });
        history.record_action(EditorAction::Insert { position: 1, character: 'i' });
        history.end_group();

        assert!(history.can_undo());
        // After grouping, only the last action of the group should be in the stack
        let (undo_count, _) = history.get_stats();
        assert_eq!(undo_count, 1);
    }

    #[test]
    fn test_capacity_limit() {
        let mut stack = UndoRedoStack::with_capacity(2);

        stack.save_state(1);
        stack.save_state(2);
        stack.save_state(3); // Should remove the first state

        assert_eq!(stack.len(), 2);

        // Should only be able to undo to state 2, not state 1
        assert_eq!(stack.undo(), Some(2));
        assert_eq!(stack.undo(), None); // No more states
    }
}