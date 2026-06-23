---@diagnostic disable: undefined-global
local call = redis.call
local now = call("TIME")[1]

local function reap(queue_key, processing_key)
	local expired = call("ZRANGEBYSCORE", processing_key, "-inf", now)

	for _, job in ipairs(expired) do
		call("LPUSH", queue_key, job)
		call("ZREM", processing_key, job)
	end

	return #expired
end

local queue_key = KEYS[1]
local processing_key = KEYS[2]

return reap(queue_key, processing_key)
