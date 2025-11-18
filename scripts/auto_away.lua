-- Auto-away script for RustIRC
-- Automatically sets away status after idle time

local idle_threshold = 300 -- 5 minutes in seconds
local idle_time = 0
local is_away = false
local away_message = "Auto-away: Idle"

-- Check idle time every minute
function check_idle()
    idle_time = idle_time + 60

    if idle_time >= idle_threshold and not is_away then
        irc.away(away_message)
        irc.print("Auto-away activated after " .. idle_threshold .. " seconds of idle time")
        is_away = true
    end
end

-- Reset idle counter on message
function irc.on_message(event)
    if event.type == "message" then
        idle_time = 0
        if is_away then
            irc.away(nil) -- Unset away
            irc.print("Auto-away deactivated - you're back!")
            is_away = false
        end
    end
end

-- Custom command to configure
irc.commands = irc.commands or {}
irc.commands.autoaway = function(args)
    if #args > 0 then
        idle_threshold = tonumber(args[1]) or idle_threshold
        irc.print("Auto-away threshold set to " .. idle_threshold .. " seconds")
    else
        irc.print("Current auto-away threshold: " .. idle_threshold .. " seconds")
        irc.print("Usage: /autoaway <seconds>")
    end
end

irc.print("Auto-away script loaded. Threshold: " .. idle_threshold .. " seconds")
irc.print("Use /autoaway <seconds> to change the threshold")
