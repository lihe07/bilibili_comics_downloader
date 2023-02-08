pub fn make_groups(list: Vec<&EpisodeCache>, num: usize) -> Vec<Item> {
    let mut groups = Vec::new();
    let mut group = Vec::new();
    for item in list {
        group.push(item);
        if group.len() == num {
            groups.push(Item::Group(group));
            group = Vec::new();
        }
    }
    if !group.is_empty() {
        groups.push(Item::Group(group));
    }
    groups
}

trait HasOrd {
    fn ord(&self) -> f64;
}

pub fn apply_range<T: HasOrd>(mut list: Vec<T>, range: &str) -> Vec<T> {
    // 范围的格式是a-b,a-,-b,c

    if list.is_empty() || range.is_empty() {
        return list;
    }

    let last_ord = list.last().unwrap().ord();

    let fragments = range.split(',');
    let mut result = Vec::new();

    for fragment in fragments {
        let mut parts = fragment.split('-');
        let from = parts.next().unwrap().parse::<f64>().unwrap_or(0.0);
        let to = parts.next().unwrap().parse::<f64>().unwrap_or(last_ord);
        if from > to {
            continue;
        }

        for item in &list {
            if item.ord() >= from && item.ord() <= to {
                if result.contains(&item.ord()) {
                    continue;
                };
                result.push(item.ord());
            }
        }
    }
    list.retain(|item| result.contains(&item.ord()));
    list
}

pub fn parse_id(s: String) -> Option<u32> {
    if s.parse::<u32>().is_ok() {
        return Some(s.parse::<u32>().unwrap());
    }
    if s.contains("mc") {
        let id = s.split("mc").collect::<Vec<&str>>()[1];
        let id = id
            .chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>();
        if let Ok(id) = id.parse::<u32>() {
            return Some(id);
        }
    }
    None
}

pub fn bytes_with_unit(bytes: u64) -> String {
    let mut bytes = bytes as f64;
    let mut unit = "B";
    if bytes > 1024. {
        bytes /= 1024.;
        unit = "KB";
    }
    if bytes > 1024. {
        bytes /= 1024.;
        unit = "MB";
    }
    if bytes > 1024. {
        bytes /= 1024.;
        unit = "GB";
    }
    if bytes > 1024. {
        bytes /= 1024.;
        unit = "TB";
    }
    format!("{} {:.4}", bytes, unit)
}
