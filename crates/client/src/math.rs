use glam::Vec2;

pub fn intersect(p1: &Vec2, p2: &Vec2, p3: &Vec2, p4: &Vec2) -> Option<Vec2> {
    if (f32::max(p1.x, p2.x)) < f32::min(p3.x, p4.x)
        || (f32::max(p1.y, p2.y)) < f32::min(p3.y, p4.y)
        || (f32::max(p3.x, p4.x)) < f32::min(p1.x, p2.x)
        || (f32::max(p3.y, p4.y)) < f32::min(p1.y, p2.y)
    {
        return None;
    }

    if (((p1.x - p3.x) * (p4.y - p3.y) - (p1.y - p3.y) * (p4.x - p3.x))
        * ((p2.x - p3.x) * (p4.y - p3.y) - (p2.y - p3.y) * (p4.x - p3.x)))
        > 0.0
        || (((p3.x - p1.x) * (p2.y - p1.y) - (p3.y - p1.y) * (p2.x - p1.x))
            * ((p4.x - p1.x) * (p2.y - p1.y) - (p4.y - p1.y) * (p2.x - p1.x)))
            > 0.0
    {
        return None;
    }

    let base_x = (p4.x - p3.x) * (p1.y - p2.y) - (p2.x - p1.x) * (p3.y - p4.y);
    if base_x == 0.0 {
        return None;
    }
    let x = ((p1.y - p3.y) * (p2.x - p1.x) * (p4.x - p3.x) + p3.x * (p4.y - p3.y) * (p2.x - p1.x)
        - p1.x * (p2.y - p1.y) * (p4.x - p3.x))
        / base_x;

    let base_y = (p1.x - p2.x) * (p4.y - p3.y) - (p2.y - p1.y) * (p3.x - p4.x);
    if base_y == 0.0 {
        return None;
    }
    let y = (p2.y * (p1.x - p2.x) * (p4.y - p3.y) + (p4.x - p2.x) * (p4.y - p3.y) * (p1.y - p2.y)
        - p4.y * (p3.x - p4.x) * (p2.y - p1.y))
        / base_y;

    Some(Vec2::new(x, y))
}