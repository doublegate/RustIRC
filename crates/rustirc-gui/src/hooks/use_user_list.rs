//! Hook for managing IRC user lists and operations

use crate::context::{IrcState, UserInfo};
use dioxus::prelude::*;
use std::collections::HashMap;

/// User list management hook
#[allow(non_snake_case)]
pub fn use_user_list(server_id: String, channel: String) -> UserListHook {
    let irc_state = use_context::<IrcState>();

    UserListHook {
        irc_state,
        server_id,
        channel,
    }
}

/// User list hook interface
pub struct UserListHook {
    pub irc_state: IrcState,
    pub server_id: String,
    pub channel: String,
}

impl UserListHook {
    /// Get all users in the channel
    pub fn get_users(&self) -> HashMap<String, UserInfo> {
        let connections = self.irc_state.connections.read();

        if let Some(connection) = connections.get(&self.server_id) {
            if let Some(channel_info) = connection.channels.get(&self.channel) {
                return channel_info.users.clone();
            }
        }

        HashMap::new()
    }

    /// Get sorted user list by status and name
    pub fn get_sorted_users(&self) -> Vec<(String, UserInfo)> {
        let users = self.get_users();
        let mut user_list: Vec<(String, UserInfo)> = users.into_iter().collect();

        // Sort by status (ops first) then alphabetically
        user_list.sort_by(|(nick_a, user_a), (nick_b, user_b)| {
            let status_a = self.get_user_status_priority(user_a);
            let status_b = self.get_user_status_priority(user_b);

            // First sort by status priority (lower number = higher priority)
            match status_a.cmp(&status_b) {
                std::cmp::Ordering::Equal => {
                    // Then sort alphabetically, case-insensitive
                    nick_a.to_lowercase().cmp(&nick_b.to_lowercase())
                }
                other => other,
            }
        });

        user_list
    }

    /// Get users grouped by status
    pub fn get_users_by_status(&self) -> UsersByStatus {
        let users = self.get_sorted_users();
        let mut result = UsersByStatus::default();

        for (nickname, user_info) in users {
            if user_info.modes.contains(&'o') {
                result.operators.push((nickname, user_info));
            } else if user_info.modes.contains(&'h') {
                result.half_ops.push((nickname, user_info));
            } else if user_info.modes.contains(&'v') {
                result.voiced.push((nickname, user_info));
            } else {
                result.regular.push((nickname, user_info));
            }
        }

        result
    }

    /// Get user count
    pub fn get_user_count(&self) -> usize {
        self.get_users().len()
    }

    /// Get specific user info
    pub fn get_user_info(&self, nickname: &str) -> Option<UserInfo> {
        let users = self.get_users();
        users.get(nickname).cloned()
    }

    /// Check if user has specific mode
    pub fn user_has_mode(&self, nickname: &str, mode: char) -> bool {
        if let Some(user) = self.get_user_info(nickname) {
            user.modes.contains(&mode)
        } else {
            false
        }
    }

    /// Check if user is operator
    pub fn is_operator(&self, nickname: &str) -> bool {
        self.user_has_mode(nickname, 'o')
    }

    /// Check if user has voice
    pub fn has_voice(&self, nickname: &str) -> bool {
        self.user_has_mode(nickname, 'v')
    }

    /// Check if user is away
    pub fn is_away(&self, nickname: &str) -> bool {
        if let Some(user) = self.get_user_info(nickname) {
            user.away
        } else {
            false
        }
    }

    /// Search users by nickname
    pub fn search_users(&self, query: &str) -> Vec<(String, UserInfo)> {
        let users = self.get_users();
        let query_lower = query.to_lowercase();

        users
            .into_iter()
            .filter(|(nickname, _)| nickname.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get users matching a pattern (for tab completion)
    pub fn get_matching_users(&self, prefix: &str) -> Vec<String> {
        let users = self.get_users();
        let prefix_lower = prefix.to_lowercase();

        let mut matches: Vec<String> = users
            .keys()
            .filter(|nickname| nickname.to_lowercase().starts_with(&prefix_lower))
            .cloned()
            .collect();

        matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        matches
    }

    /// Add user to channel
    pub fn add_user(&self, nickname: String, modes: std::collections::HashSet<char>) {
        if let Some(connection) = self.irc_state.connections.write().get_mut(&self.server_id) {
            if let Some(channel_info) = connection.channels.get_mut(&self.channel) {
                let user_info = UserInfo {
                    nickname: nickname.clone(),
                    modes,
                    away: false,
                    realname: None,
                };
                channel_info.users.insert(nickname, user_info);
            }
        }
    }

    /// Remove user from channel
    pub fn remove_user(&self, nickname: &str) {
        if let Some(connection) = self.irc_state.connections.write().get_mut(&self.server_id) {
            if let Some(channel_info) = connection.channels.get_mut(&self.channel) {
                channel_info.users.remove(nickname);
            }
        }
    }

    /// Update user mode
    pub fn update_user_mode(&self, nickname: &str, mode: char, add: bool) {
        if let Some(connection) = self.irc_state.connections.write().get_mut(&self.server_id) {
            if let Some(channel_info) = connection.channels.get_mut(&self.channel) {
                if let Some(user_info) = channel_info.users.get_mut(nickname) {
                    if add {
                        user_info.modes.insert(mode);
                    } else {
                        user_info.modes.remove(&mode);
                    }
                }
            }
        }
    }

    /// Update user away status
    pub fn update_user_away_status(&self, nickname: &str, away: bool) {
        if let Some(connection) = self.irc_state.connections.write().get_mut(&self.server_id) {
            if let Some(channel_info) = connection.channels.get_mut(&self.channel) {
                if let Some(user_info) = channel_info.users.get_mut(nickname) {
                    user_info.away = away;
                }
            }
        }
    }

    /// Rename user (nick change)
    pub fn rename_user(&self, old_nick: &str, new_nick: String) {
        if let Some(connection) = self.irc_state.connections.write().get_mut(&self.server_id) {
            if let Some(channel_info) = connection.channels.get_mut(&self.channel) {
                if let Some(mut user_info) = channel_info.users.remove(old_nick) {
                    user_info.nickname = new_nick.clone();
                    channel_info.users.insert(new_nick, user_info);
                }
            }
        }
    }

    /// Get user status priority for sorting (lower = higher priority)
    fn get_user_status_priority(&self, user: &UserInfo) -> u8 {
        if user.modes.contains(&'o') {
            0 // Operator
        } else if user.modes.contains(&'h') {
            1 // Half-op
        } else if user.modes.contains(&'v') {
            2 // Voice
        } else {
            3 // Regular user
        }
    }

    /// Get user status symbol
    pub fn get_user_status_symbol(&self, user: &UserInfo) -> &'static str {
        if user.modes.contains(&'o') {
            "@" // Operator
        } else if user.modes.contains(&'h') {
            "%" // Half-op
        } else if user.modes.contains(&'v') {
            "+" // Voice
        } else {
            "" // Regular user
        }
    }

    /// Get user display name with status symbol
    pub fn get_user_display_name(&self, nickname: &str, user: &UserInfo) -> String {
        let symbol = self.get_user_status_symbol(user);
        format!("{}{}", symbol, nickname)
    }
}

/// Users grouped by status
#[derive(Debug, Default)]
pub struct UsersByStatus {
    pub operators: Vec<(String, UserInfo)>,
    pub half_ops: Vec<(String, UserInfo)>,
    pub voiced: Vec<(String, UserInfo)>,
    pub regular: Vec<(String, UserInfo)>,
}

impl UsersByStatus {
    /// Get total user count across all categories
    pub fn total_count(&self) -> usize {
        self.operators.len() + self.half_ops.len() + self.voiced.len() + self.regular.len()
    }

    /// Check if any category has users
    pub fn is_empty(&self) -> bool {
        self.total_count() == 0
    }
}

/// Hook for user list filtering and search
#[allow(non_snake_case)]
pub fn use_user_list_filter(server_id: String, channel: String) -> UserListFilterHook {
    let mut search_query = use_signal(|| String::new());
    let mut show_away_users = use_signal(|| true);
    let mut group_by_status = use_signal(|| true);

    let user_list = use_user_list(server_id, channel);

    UserListFilterHook {
        user_list,
        search_query,
        show_away_users,
        group_by_status,
    }
}

/// User list filter hook interface
pub struct UserListFilterHook {
    pub user_list: UserListHook,
    pub search_query: Signal<String>,
    pub show_away_users: Signal<bool>,
    pub group_by_status: Signal<bool>,
}

impl UserListFilterHook {
    /// Get filtered user list
    pub fn get_filtered_users(&self) -> Vec<(String, UserInfo)> {
        let mut users = if self.search_query.read().is_empty() {
            self.user_list.get_sorted_users()
        } else {
            self.user_list.search_users(&self.search_query.read())
        };

        // Filter out away users if disabled
        if !self.show_away_users() {
            users.retain(|(_, user)| !user.away);
        }

        users
    }

    /// Get filtered users grouped by status
    pub fn get_filtered_users_by_status(&self) -> UsersByStatus {
        if !self.group_by_status() {
            // Return all users in regular category if grouping is disabled
            let users = self.get_filtered_users();
            let mut result = UsersByStatus::default();
            result.regular = users;
            return result;
        }

        let users = self.get_filtered_users();
        let mut result = UsersByStatus::default();

        for (nickname, user_info) in users {
            if user_info.modes.contains(&'o') {
                result.operators.push((nickname, user_info));
            } else if user_info.modes.contains(&'h') {
                result.half_ops.push((nickname, user_info));
            } else if user_info.modes.contains(&'v') {
                result.voiced.push((nickname, user_info));
            } else {
                result.regular.push((nickname, user_info));
            }
        }

        result
    }

    /// Set search query
    pub fn set_search_query(&self, query: String) {
        self.search_query.set(query);
    }

    /// Toggle show away users
    pub fn toggle_show_away_users(&self) {
        self.show_away_users.set(!self.show_away_users());
    }

    /// Toggle group by status
    pub fn toggle_group_by_status(&self) {
        self.group_by_status.set(!self.group_by_status());
    }

    /// Clear search
    pub fn clear_search(&self) {
        self.search_query.set(String::new());
    }
}
