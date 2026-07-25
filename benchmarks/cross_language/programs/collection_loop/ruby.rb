values = []
50_000.times do |i|
  values << (i % 997)
end
total = 0
values.each do |value|
  total += value
end
puts total
