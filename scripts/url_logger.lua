-- url_logger.lua
-- @description: Logs URLs posted in channels with timestamps
-- @author: RustIRC Contributors
-- @version: 1.0.0

---[[
URL Logger Script for RustIRC

Features:
- Detects URLs in chat messages
- Stores URLs with timestamps and channel info
- Provides commands to view recent URLs
- Supports filtering by channel
- Configurable buffer size

Usage:
  /urls [count]           - Show last N URLs (default: 10)
  /urls clear             - Clear URL log
  /urls search <text>     - Search URLs containing text

Configuration:
  Edit the CONFIG table below to customize behavior
]]---

-- Configuration
local CONFIG = {
    enabled = true,
    max_urls = 500,              -- Maximum URLs to store
    show_notifications = false,   -- Notify on URL detection
    excluded_channels = {},       -- Channels to ignore
}

-- State
local url_log = {}

-- URL pattern (basic HTTP/HTTPS detection)
local URL_PATTERN = "https?://[%w-_%.%?:/%+=&%%#~]+"

---Helper function to add URL to log
---@param url string The URL to log
---@param channel string Channel where URL was posted
---@param nick string Nickname who posted it
local function log_url(url, channel, nick)
    local entry = {
        url = url,
        channel = channel,
        nick = nick,
        timestamp = os.time(),
        date = os.date("%Y-%m-%d %H:%M:%S"),
    }

    table.insert(url_log, entry)

    -- Maintain buffer size
    if #url_log > CONFIG.max_urls then
        table.remove(url_log, 1)
    end

    -- Optional notification
    if CONFIG.show_notifications then
        irc.notify("URL Posted", url)
    end

    irc.log("debug", string.format("Logged URL from %s in %s: %s", nick, channel, url))
end

---Check if channel is excluded
---@param channel string Channel to check
---@return boolean
local function is_excluded(channel)
    for _, excluded in ipairs(CONFIG.excluded_channels) do
        if channel == excluded then
            return true
        end
    end
    return false
end

---Format URL entry for display
---@param entry table URL log entry
---@return string
local function format_entry(entry)
    return string.format("[%s] <%s/%s> %s",
        entry.date,
        entry.channel,
        entry.nick,
        entry.url
    )
end

-- Event Handler: Message received
function irc.on_message(event)
    if not CONFIG.enabled then return end
    if event.type ~= "message" then return end
    if not event.params or #event.params < 2 then return end

    local channel = event.params[1]
    local message = event.params[#event.params]

    -- Skip excluded channels
    if is_excluded(channel) then return end

    local nick = "unknown" -- TODO: Extract from IRC message prefix if available
    -- Nickname extraction not currently implemented

    -- Find all URLs in message
    for url in message:gmatch(URL_PATTERN) do
        log_url(url, channel, nick)
    end
end

-- Custom Commands
irc.commands = irc.commands or {}

---Command: View recent URLs
irc.commands.urls = function(args)
    local subcmd = args[1] or "list"

    if subcmd == "clear" then
        -- Clear URL log
        local count = #url_log
        url_log = {}
        irc.print(string.format("Cleared %d URLs from log", count))
        return
    end

    if subcmd == "search" then
        -- Search URLs
        if not args[2] then
            irc.print("Usage: /urls search <text>")
            return
        end

        local search_term = args[2]:lower()
        local matches = {}

        for _, entry in ipairs(url_log) do
            if entry.url:lower():find(search_term, 1, true) then
                table.insert(matches, entry)
            end
        end

        if #matches == 0 then
            irc.print("No URLs found matching: " .. args[2])
        else
            irc.print(string.format("Found %d URLs matching '%s':", #matches, args[2]))
            for _, entry in ipairs(matches) do
                irc.echo(format_entry(entry))
            end
        end
        return
    end

    -- Default: List recent URLs
    local count = tonumber(subcmd) or 10
    count = math.min(count, #url_log)

    if #url_log == 0 then
        irc.print("No URLs logged yet")
        return
    end

    irc.print(string.format("Last %d URLs (total: %d):", count, #url_log))

    local start = math.max(#url_log - count + 1, 1)
    for i = start, #url_log do
        irc.echo(format_entry(url_log[i]))
    end
end

---Command: Configure URL logger
irc.commands.urlconfig = function(args)
    if #args == 0 then
        -- Show current configuration
        irc.print("URL Logger Configuration:")
        irc.echo("  enabled: " .. tostring(CONFIG.enabled))
        irc.echo("  max_urls: " .. CONFIG.max_urls)
        irc.echo("  notifications: " .. tostring(CONFIG.show_notifications))
        irc.echo("  current URLs: " .. #url_log)
        return
    end

    local setting = args[1]:lower()
    local value = args[2]

    if setting == "enable" then
        CONFIG.enabled = true
        irc.print("URL logging enabled")
    elseif setting == "disable" then
        CONFIG.enabled = false
        irc.print("URL logging disabled")
    elseif setting == "notifications" then
        CONFIG.show_notifications = value == "on" or value == "true"
        irc.print("Notifications: " .. tostring(CONFIG.show_notifications))
    elseif setting == "maxurls" and value then
        local new_max = tonumber(value)
        if new_max and new_max > 0 then
            CONFIG.max_urls = new_max
            irc.print("Max URLs set to: " .. new_max)

            -- Trim if needed
            while #url_log > CONFIG.max_urls do
                table.remove(url_log, 1)
            end
        else
            irc.print("Invalid number for maxurls")
        end
    else
        irc.print("Usage: /urlconfig [enable|disable|notifications on/off|maxurls <num>]")
    end
end

-- Initialization
irc.print("URL Logger v1.0.0 loaded")
irc.print("Commands: /urls [count|clear|search], /urlconfig")
irc.log("info", string.format("URL Logger initialized (max: %d URLs)", CONFIG.max_urls))
