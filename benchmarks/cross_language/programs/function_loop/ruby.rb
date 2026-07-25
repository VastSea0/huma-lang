def mix(state, index)
  (state * 48_271 + index) % 2_147_483_647
end

state = 1
(1..100_000).each do |i|
  state = mix(state, i)
end
puts state
