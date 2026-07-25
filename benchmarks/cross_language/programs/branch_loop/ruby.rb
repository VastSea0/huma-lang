state = 1
total = 0
(1..200_000).each do |i|
  state = state * 2 + i
  state -= 1_000_000_000 if state >= 1_000_000_000
  state -= 1_000_000_000 if state >= 1_000_000_000
  total += state
end
puts total
