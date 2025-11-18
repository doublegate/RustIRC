-- Highlight script for RustIRC
-- Highlights messages containing specified keywords

local highlight_words = {"important", "urgent", "help"}
local highlight_users = {}

-- Check message for highlights
function irc.on_message(event)
    if event.type == "message" and event.params and #event.params >= 2 then
        local message = event.params[#event.params]:lower()
        local nick = event.params[1]

        -- Check for keyword highlights
        for _, word in ipairs(highlight_words) do
            if string.find(message, word:lower()) then
                irc.notify("Highlight", "Message contains: " .. word)
                irc.beep()
                irc.log("info", "Highlighted message from " .. nick .. ": " .. message)
                return
            end
        end

        -- Check for user highlights
        for _, user in ipairs(highlight_users) do
            if nick:lower() == user:lower() then
                irc.notify("Highlight", "Message from " .. user)
                irc.beep()
                return
            end
        end
    end
end

-- Custom commands
irc.commands = irc.commands or {}

irc.commands.highlight = function(args)
    if #args > 0 then
        local word = args[1]:lower()
        table.insert(highlight_words, word)
        irc.print("Added highlight word: " .. word)
    else
        irc.print("Highlight words: " .. table.concat(highlight_words, ", "))
        irc.print("Usage: /highlight <word>")
    end
end

irc.commands.unhighlight = function(args)
    if #args > 0 then
        local word = args[1]:lower()
        for i, w in ipairs(highlight_words) do
            if w == word then
                table.remove(highlight_words, i)
                irc.print("Removed highlight word: " .. word)
                return
            end
        end
        irc.print("Word not found in highlight list: " .. word)
    else
        irc.print("Usage: /unhighlight <word>")
    end
end

irc.commands.highlightuser = function(args)
    if #args > 0 then
        local user = args[1]
        table.insert(highlight_users, user)
        irc.print("Added highlight user: " .. user)
    else
        irc.print("Highlighted users: " .. table.concat(highlight_users, ", "))
        irc.print("Usage: /highlightuser <nick>")
    end
end

irc.print("Highlight script loaded")
irc.print("Commands: /highlight, /unhighlight, /highlightuser")
