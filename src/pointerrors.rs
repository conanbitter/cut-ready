use crate::geometry::Point2D;

const CONNECT_TRESHOLD: f32 = 0.0001;
const CONNECT_TRESHOLD2: f32 = CONNECT_TRESHOLD * CONNECT_TRESHOLD;

pub fn collect_errors(open_ends: &[Point2D]) -> Vec<Point2D> {
    let mut removed = vec![false; open_ends.len()];

    for (i, p1) in open_ends.iter().enumerate() {
        if removed[i] {
            continue;
        }

        for (j, p2) in open_ends.iter().enumerate().skip(i + 1) {
            if removed[j] {
                continue;
            }

            if p1.dist2(*p2) < CONNECT_TRESHOLD2 {
                removed[i] = true;
                removed[j] = true;
                break;
            }
        }
    }

    open_ends
        .iter()
        .zip(removed.iter())
        .filter_map(|(&point, &is_removed)| if is_removed { None } else { Some(point) })
        .collect()
}
