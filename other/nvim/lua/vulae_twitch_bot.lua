local function execute_command(message)
	local args = vim.split(message, " ")
	if args[1] == "set_theme" then
		vim.cmd.colorscheme(args[2] or "habamax")
	end
end

local function start_client()
	vim.fn.sockconnect("tcp", "127.0.0.1:24694", {
		on_data = function(_channel_id, data)
			for _, message in pairs(data) do
				execute_command(message)
			end
		end,
	})
end

pcall(start_client)
