--os.exit(0)
function dump(o)
   if type(o) == 'table' then
      local s = '{ '
      for k,v in pairs(o) do
         if type(k) ~= 'number' then k = '"'..k..'"' end
         s = s .. '['..k..'] = ' .. dump(v) .. ','
      end
      return s .. '} '
   else
      return tostring(o)
   end
end
--

print("Global vars only for reading not for change")
print(BOT_TG_FATHER_KEY)
print(BOT_TG_USERID)
print(ALERT_DESKTOP_ENABLED)
print(LERT_TELEGRAM_ENABLED)
print(CHARTS_URL)
print(GEMINI_API_KEY)
print(PROXY_URL)
print(LANGUAGE_AI)
print("End global vars")

print("Dump detector loaded!")
while true do 

--print("active_pairs1=", active_pairs[1])
local data = get_market_data(active_pairs[1])
--print( dump(data) )
if data and #data > 2 then
    local last = data[#data].bid_price[1]
    local prev = data[#data-1].bid_price[1]
    print("last1:", last)
    print("prev1:", prev)
    if last < prev * 1.05 then
        send_to_rust_alert("LUA DETECTED DUMP: ".. active_pairs[1] .. " price jumped to " .. last)
    else
	    --send_to_rust_alert("just a test")
    end
end
--sleep(200) -- sleep blocking main thread, not thread of script
end

--while true do
--	send_to_rust_alert("JUST A TEST")
--	sleep(1000)
--end
