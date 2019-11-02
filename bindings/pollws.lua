local ffi = ffi or _G.ffi or require("ffi")
ffi.cdef[[
struct pollsocket* pollws_open(const char* url);
void pollws_close(struct pollsocket* ctx);
int pollws_status(struct pollsocket* ctx);
void pollws_send(struct pollsocket* ctx, const char* msg);
int pollws_poll(struct pollsocket* ctx);
unsigned int pollws_get(struct pollsocket* ctx, char* dest, unsigned int dest_size);
unsigned int pollws_pop(struct pollsocket* ctx, char* dest, unsigned int dest_size);
]]

local pollws = ffi.load("pollws")

local socket_mt = {}
function socket_mt:open(url, scratch_size)
  if self._socket then self:close() end
  if not scratch_size then scratch_size = 64000 end
  self._socket = pollws.pollws_open(url),
  self._scratch = ffi.new("int8_t[?]", scratch_size),
  self._scratch_size = scratch_size
end
function socket_mt:poll()
  if not self._socket then return end
  local msg_size = pollws.pollws_pop(self._socket, self._scratch, self._scratch_size)
  if msg_size > 0 then
    local smsg = ffi.string(self._scratch, msg_size)
    return smsg
  else
    return nil
  end
end
function socket_mt:send(msg)
  if not self._socket then return end
  pollws.pollws_send(self._socket, msg)
end
function socket_mt:close()
  pollws.pollws_close(self._socket)
  self._socket = nil
end

local function open(url, scratch_size)
  local socket = setmetatable({}, {__index = socket_mt})
  socket:open(url, scratch_size)
  return socket
end

return {open = open, pollws = pollws}