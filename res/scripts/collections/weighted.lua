--- A multiset of weight-value pairs for weighted random selection.
--- @class WeightedBag
--- @field package root WeightedBagNode
--- @field package count integer
local C_WeightedBag, new_WeightedBag, M_WeightedBag = class 'WeightedBag'

--- @class WeightedBagNode
--- @field parent WeightedBagNode?
--- @field weight number
--- @field left WeightedBagNode?
--- @field right WeightedBagNode?
--- @field value any?
local C_WeightedBagNode, new_WeightedBagNode = class 'WeightedBagNode'

--- @param weight any
local function normalize_weight(weight)
    if not weight then return 0 end
    if not is_number(weight) or weight == math.huge then return 1 end
    if is_nan(weight) or weight < 0 then return 0 end
    return weight
end

--- @param parent WeightedBagNode?
--- @param left WeightedBagNode?
--- @param right WeightedBagNode?
--- @param value any?
--- @param weight number?
local function WeightedBagNode(parent, left, right, value, weight)
    --- @type WeightedBagNode
    local instance = {
        parent = parent,
        weight = weight or 0.0,
        left = left,
        right = right,
        value = value
    }
    return new_WeightedBagNode(instance)
end

function C_WeightedBagNode:is_leaf()
    return not self.left and not self.right
end

function C_WeightedBagNode:try_insert(value, weight)
    weight = normalize_weight(weight)

    if self:is_leaf() then
        -- Promote leaf to branch
        local value_a = self.value
        local weight_a = normalize_weight(self.weight)
        self.left = WeightedBagNode(self, nil, nil, self.value, self.weight)
        self.right = WeightedBagNode(self, nil, nil, value, weight)
        self.value = nil
    else
        -- Find empty space on branch to insert
        if not self.left then
            self.left = WeightedBagNode(self, nil, nil, value, weight)
        elseif not self.right then
            self.right = WeightedBagNode(self, nil, nil, value, weight)
        else
            return false
        end
    end

    -- Propagate weight change to parents
    local current_branch = self
    repeat
        current_branch.weight = current_branch.weight + weight
        current_branch = current_branch.parent
    until not current_branch
    return true
end

--- Creates a new `WeightedBag` instance.
--- @return WeightedBag
function WeightedBag()
    --- @type WeightedBag
    local instance = {
        root = WeightedBagNode(),
        count = 0
    }
    return new_WeightedBag(instance)
end

--- Finds the next node to insert into.
--- @param root WeightedBagNode?
--- @return WeightedBagNode?
function get_next_insert_node(root)
    local current_node = root
    while current_node do
        if current_node:is_leaf() then break end
        local node_left = current_node.left
        local node_right = current_node.right
        if node_left and node_right then
            if node_left.weight <= node_right.weight then
                current_node = node_left
            else
                current_node = node_right
            end
        else
            current_node = node_left or node_right
        end
    end
    return current_node
end

--- Adds a new element to the bag. 
--- If no weight is specified, defaults to 1.
--- @param value any? @ The element to add.
--- @param weight? number @ The element's weight value.
function C_WeightedBag:add(value, weight)
    local lightest = get_next_insert_node(self.root)
    if lightest then
       lightest:try_insert(value, weight)
    end
    self.count = self.count + 1
end

--- Performs a weighted random selection from the bag's elements and returns the selected element.
--- @param rng? Rng
--- @return any?
--- @overload fun(): any?
function C_WeightedBag:next(rng)
    rng = rng or GLOBAL_RNG
    --- @type WeightedBagNode?
    local current_node = self.root
    local current_choice = rng:float(0, self.root.weight)
    while current_node and not current_node:is_leaf() do
        local left = current_node.left
        local right = current_node.right
        if left and right then
            if current_choice > left.weight then
                current_node = right
                current_choice = current_choice - left.weight
            else
                current_node = left
            end
        else
            current_node = left or right
        end
    end
    return current_node and current_node.value
end

--- @param bag WeightedBag
function M_WeightedBag.__len(bag)
    return bag.count
end