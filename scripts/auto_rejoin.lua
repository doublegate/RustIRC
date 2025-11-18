-- Auto-rejoin script for RustIRC
-- Automatically rejoins channels after being kicked

local auto_rejoin_enabled = true
local rejoin_delay = 3 -- seconds
local rejoin_channels = {}

-- Handle kick events
function irc.on_user_part(event)
    if event.type == "user_part" then
        local channel = event.channel
        local user = event.user
        local my_nick = irc.my_nick()

        if user == my_nick and auto_rejoin_enabled then
            irc.log("info", "Kicked from " .. channel .. ", will rejoin in " .. rejoin_delay .. " seconds")

            -- Store channel for rejoin
            table.insert(rejoin_channels, channel)

            -- Note: In real implementation, would use timer
            irc.join(channel)
            irc.print("Rejoining " .. channel)
        end
    end
end

-- Custom commands
irc.commands = irc.commands or {}

irc.commands.autorejoin = function(args)
    if #args > 0 then
        local cmd = args[1]:lower()
        if cmd == "on" then
            auto_rejoin_enabled = true
            irc.print("Auto-rejoin enabled")
        elseif cmd == "off" then
            auto_rejoin_enabled = false
            irc.print("Auto-rejoin disabled")
        elseif cmd == "delay" and #args > 1 then
            rejoin_delay = tonumber(args[2]) or rejoin_delay
            irc.print("Rejoin delay set to " .. rejoin_delay .. " seconds")
        else
            irc.print("Usage: /autorejoin [on|off|delay <seconds>]")
        end
    else
        local status = auto_rejoin_enabled and "enabled" or "disabled"
        irc.print("Auto-rejoin is " .. status .. " (delay: " .. rejoin_delay .. "s)")
    end
end

irc.print("Auto-rejoin script loaded")
irc.print("Status: " .. (auto_rejoin_enabled and "enabled" or "disabled"))
irc.print("Use /autorejoin [on|off|delay <seconds>]")
