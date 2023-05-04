### lab3

#### 实现功能总结
1. spawn系统调用， 结合fork和exec调用修改即可
2. stride 调度算法， 在任务的fetch的时候选取步长最小的任务并调整步长即可
#### 问答题
stride 算法深入
1. 实际情况是轮到 p1 执行吗？为什么？ 不是，因为u8会导致整数溢出，会导致p2的步长小于p1，所以此时轮到p2执行
2. 证明： 当优先级大于等于2时， pass必定小于等于BigStride / 2， 设stride_max = x * pass1 <= x * bigStride /2, stride_min同理，整理不等式，stride_max - stride_min <= bigstride * (x - y) / 2, 因为x >= y, 得证.
3. 补全代码： 
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ovflow = !(stride + pass < u8::MAX);
        let ovflowother = !(other.stride + pass < u8::MAX);
        let twoovflow =  ovflow && ovflowother;
        assert(self.stride < bigstride / 2);
        assert(other.stride < bigstride / 2);
        if abs(self.stride - other.stride) < bigstride / 2 && twoovflow{
            // 说明未溢出，正常比较
            if self.stride < other.stride {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater) 
            }
        } else {
            // 说明溢出，比较取反
            // 包含两个都溢出的情况和只有一个溢出的情况
            // 两个都溢出明显取小值， 只有一个溢出明显取未溢出的值，都为取反
            if self.stride > other.stride {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater) 
            }
        }
        

    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
#### 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    无

2. 本次实验未参考实验指导书以外的其他资料

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。