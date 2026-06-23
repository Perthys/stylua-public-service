---@diagnostic disable: undefined-global
local call = redis.call
local now = call("TIME")[1]

local function consume(queue_key, processing_key, visibility)
	local job = call("RPOP", queue_key)

	if job then
		call("ZADD", processing_key, now + tonumber(visibility), job)
	end

	return job
end

local queue_key = KEYS[1]
local processing_key = KEYS[2]
local visibility = ARGV[1]

return consume(queue_key, processing_key, visibility)
