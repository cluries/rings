-- 名字候选列表
local nameList = {
    "张三", "李四", "王五", "赵六", "孙七", "周八", "吴九", "郑十",
    "陈一", "钱二", "孟三", "王四", "李五", "周六", "吴七", "郑八",
}

local counter = 0

-- 根据年龄和性别生成名字的函数
-- @param age 年龄
-- @param gender 性别 ('M' 表示男性, 'F' 表示女性)
-- @return string 生成的名字
function generateName(age, gender)
    -- 将年龄和性别组合成字符串进行hash
    local input = age .. gender
    local hash = 0
    
    -- 简单的字符串hash算法
    for i = 1, #input do
        hash = hash * 31 + string.byte(input, i)
    end
    
    -- 确保hash值为正数
    hash = math.abs(hash)
    
    -- 使用hash值选择名字列表中的一个名字
    local index = (hash % #nameList) + 1


    counter = counter + 1

    return "Counter:" .. counter .." | " .. age .. ":" .. nameList[index] .. test_func(100,200)
end
