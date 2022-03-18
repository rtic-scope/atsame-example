target extended-remote :3333

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

load

# start the process but immediately halt the processor
stepi

# Helpers
define reload
  monitor reset halt
  continue
end

define reflash
  monitor reset halt
  load
  continue
end

alias rl = reload
alias rf = reflash
