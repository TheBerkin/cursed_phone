ease = {}

--- @param x number
function ease.sine_in(x)
    return 1.0 - math.cos((x * math.pi) * 0.5)
end

--- @param x number
function ease.sine_out(x)
    return math.sin((x * math.pi) * 0.5)
end

--- @param x number
function ease.sine(x)
    return -(math.cos(math.pi * x)) * 0.5 + 0.5
end

--- @param x number
function ease.cubic(x)
    if x < 0.5 then
        return 4.0 * x ^ 3
    else
        return 1.0 - ((-2.0 * x + 2.0) ^ 3) * 0.5
    end
end

--- @param x number
function ease.linear(x)
    return x
end

--- @param x number
function ease.quadratic(x)
    return math.pow(x, 2)
end