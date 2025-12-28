// Command registry for CADDY CAD system
// Manages command registration, lookup, and autocomplete

use super::command::Command;
use std::collections::HashMap;

/// Registry of all available commands
pub struct CommandRegistry {
    /// Commands indexed by name
    commands: HashMap<String, Box<dyn Command>>,
    /// Aliases mapping to command names
    aliases: HashMap<String, String>,
    /// Command categories for organization
    categories: HashMap<String, Vec<String>>,
}

impl CommandRegistry {
    /// Create a new empty command registry
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
            categories: HashMap::new(),
        }
    }

    /// Register a command
    pub fn register(&mut self, command: Box<dyn Command>) {
        let name = command.name().to_uppercase();

        // Register aliases
        for alias in command.aliases() {
            let alias_upper = alias.to_uppercase();
            self.aliases.insert(alias_upper, name.clone());
        }

        // Store command
        self.commands.insert(name, command);
    }

    /// Register a command with a specific category
    pub fn register_with_category(&mut self, command: Box<dyn Command>, category: impl Into<String>) {
        let name = command.name().to_uppercase();
        let category = category.into();

        // Add to category
        self.categories.entry(category)
            .or_insert_with(Vec::new)
            .push(name.clone());

        // Register the command
        self.register(command);
    }

    /// Look up a command by name or alias
    pub fn get(&self, name: &str) -> Option<&Box<dyn Command>> {
        let name_upper = name.to_uppercase();

        // Try direct lookup first
        if let Some(cmd) = self.commands.get(&name_upper) {
            return Some(cmd);
        }

        // Try alias lookup
        if let Some(actual_name) = self.aliases.get(&name_upper) {
            return self.commands.get(actual_name);
        }

        None
    }

    /// Clone a command by name or alias
    pub fn clone_command(&self, name: &str) -> Option<Box<dyn Command>> {
        self.get(name).map(|cmd| cmd.clone_box())
    }

    /// Check if a command exists
    pub fn contains(&self, name: &str) -> bool {
        let name_upper = name.to_uppercase();
        self.commands.contains_key(&name_upper) || self.aliases.contains_key(&name_upper)
    }

    /// Get all registered command names
    pub fn command_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.commands.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all aliases
    pub fn aliases(&self) -> Vec<(String, String)> {
        self.aliases.iter()
            .map(|(alias, cmd)| (alias.clone(), cmd.clone()))
            .collect()
    }

    /// Get commands in a specific category
    pub fn get_category(&self, category: &str) -> Vec<String> {
        self.categories.get(category)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all categories
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self.categories.keys().cloned().collect();
        cats.sort();
        cats
    }

    /// Get autocomplete suggestions for a partial command name
    pub fn autocomplete(&self, partial: &str) -> Vec<String> {
        let partial_upper = partial.to_uppercase();
        let mut suggestions = Vec::new();

        // Check command names
        for name in self.commands.keys() {
            if name.starts_with(&partial_upper) {
                suggestions.push(name.clone());
            }
        }

        // Check aliases
        for (alias, cmd_name) in &self.aliases {
            if alias.starts_with(&partial_upper) {
                suggestions.push(format!("{} ({})", alias, cmd_name));
            }
        }

        suggestions.sort();
        suggestions
    }

    /// Get command help text
    pub fn get_help(&self, name: &str) -> Option<String> {
        self.get(name).map(|cmd| {
            format!(
                "{} - {}\n\nUsage: {}\n\nAliases: {}",
                cmd.name(),
                cmd.description(),
                cmd.usage(),
                cmd.aliases().join(", ")
            )
        })
    }

    /// Get all command help texts
    pub fn get_all_help(&self) -> String {
        let mut help = String::from("Available Commands:\n\n");

        for category in self.categories() {
            help.push_str(&format!("\n{}:\n", category));
            for cmd_name in self.get_category(&category) {
                if let Some(cmd) = self.commands.get(&cmd_name) {
                    help.push_str(&format!(
                        "  {:12} - {}\n",
                        cmd.name(),
                        cmd.description()
                    ));
                }
            }
        }

        // Add uncategorized commands
        let categorized: Vec<String> = self.categories.values()
            .flat_map(|cmds| cmds.clone())
            .collect();

        let uncategorized: Vec<&String> = self.commands.keys()
            .filter(|name| !categorized.contains(name))
            .collect();

        if !uncategorized.is_empty() {
            help.push_str("\nOther:\n");
            for cmd_name in uncategorized {
                if let Some(cmd) = self.commands.get(cmd_name) {
                    help.push_str(&format!(
                        "  {:12} - {}\n",
                        cmd.name(),
                        cmd.description()
                    ));
                }
            }
        }

        help
    }

    /// Count registered commands
    pub fn count(&self) -> usize {
        self.commands.len()
    }

    /// Clear all registered commands
    pub fn clear(&mut self) {
        self.commands.clear();
        self.aliases.clear();
        self.categories.clear();
    }

    /// Register standard CAD commands
    pub fn register_standard_commands(&mut self) {
        // This will be called to register all built-in commands
        // Commands are registered by individual modules
    }

    /// Remove a command by name
    pub fn unregister(&mut self, name: &str) -> bool {
        let name_upper = name.to_uppercase();

        // Remove from commands
        let removed = self.commands.remove(&name_upper).is_some();

        // Remove aliases pointing to this command
        self.aliases.retain(|_, cmd_name| cmd_name != &name_upper);

        // Remove from categories
        for cmds in self.categories.values_mut() {
            cmds.retain(|cmd_name| cmd_name != &name_upper);
        }

        removed
    }

    /// Get fuzzy matches for a command name (for typo correction)
    pub fn fuzzy_match(&self, name: &str, max_distance: usize) -> Vec<String> {
        let name_upper = name.to_uppercase();
        let mut matches = Vec::new();

        for cmd_name in self.commands.keys() {
            if levenshtein_distance(&name_upper, cmd_name) <= max_distance {
                matches.push(cmd_name.clone());
            }
        }

        // Also check aliases
        for (alias, _) in &self.aliases {
            if levenshtein_distance(&name_upper, alias) <= max_distance {
                matches.push(alias.clone());
            }
        }

        matches.sort();
        matches.dedup();
        matches
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate Levenshtein distance between two strings (for fuzzy matching)
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = std::cmp::min(
                std::cmp::min(
                    matrix[i][j + 1] + 1,     // deletion
                    matrix[i + 1][j] + 1      // insertion
                ),
                matrix[i][j] + cost           // substitution
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("LINE", "LINE"), 0);
        assert_eq!(levenshtein_distance("LINE", "LIEN"), 1);
        assert_eq!(levenshtein_distance("LINE", "CIRCLE"), 5);
    }

    #[test]
    fn test_registry_creation() {
        let registry = CommandRegistry::new();
        assert_eq!(registry.count(), 0);
    }
}
