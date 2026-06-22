---@diagnostic disable: undefined-global
local call = redis.call
local now = redis.call("TIME")[1]

local function check_if_acquirable(permits_key, ttl, limit, permit_id)
	call("ZREMRANGEBYSCORE", permits_key, "-inf", now)

	local acquirable = call("ZCARD", permits_key) < tonumber(limit)

	if acquirable then
		call("ZADD", permits_key, tonumber(now) + tonumber(ttl), permit_id)
		return 1 -- apparently boolean values are not supported in resp3 :(
	else
		return 0 -- apparently boolean values are not supported in resp3 :(
	end
end

local permits_key = KEYS[1]
local ttl = ARGV[1]
local limit = ARGV[2]
local permit_id = ARGV[3]

return check_if_acquirable(permits_key, ttl, limit, permit_id)
