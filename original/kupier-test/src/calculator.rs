use num_format::{Locale, ToFormattedString};

const MAX_ORDER: u32 = 4096;
const MAX_PAGE_SIZE: u32 = 8192;

fn calculate_max_order(
    page_size: u32,
    header_size: u32,
    key_size: u32,
    record_size: u32) -> u32 {
    let usable_space = page_size - header_size;
    // Get the number down to a realm where the calculations are quicker.
    let mut d = ((page_size / (key_size + record_size)) / 4) * 3;

    loop {
        let possible_space =
            key_size * (d - 1) + (record_size * d);

        if possible_space >= usable_space {
            let max_d = d - 1;
            let computed_space = key_size * (max_d - 1) + (record_size * max_d);

            println!("Space Available: {}, \
                      Space Used: {}, \
                      Elements Possible: {}, \
                      Unusable Space: {}, \
                      Total Element Size: {}",
                     page_size,
                     header_size + (key_size * (max_d - 1)) + (record_size * max_d),
                     max_d,
                     usable_space - computed_space,
                     key_size + record_size);

            return max_d;
        }

        d = d + 1;
    }
}

fn calculate_efficiency(page_size: u32, optimum_page_size: u32, value: u32, records: u64) {
    let percentage = page_size as f64 / optimum_page_size as f64;

    let num_pages = records as f64 / value as f64;
    let total_space = num_pages * page_size as f64;
    let wasted_space = (1.0 - percentage) * total_space;

    let total_space_gb = total_space / 1024.0 / 1024.0 / 1024.0;
    let mut wasted_space_gb = wasted_space / 1024.0 / 1024.0 / 1024.0;

    if wasted_space_gb < 0.0 {
        wasted_space_gb = 0.0;
    }

    println!("# Records: {}, \
              Total: {:.2} GB, \
              Wasted: {:.2} GB, \
              Efficiency: {:.2}",
             records.to_formatted_string(&Locale::en),
             total_space_gb,
             wasted_space_gb,
             percentage)
}

fn calculate_page_size(
    max_order: u32,
    header_size: u32,
    key_size: u32,
    file_offset_size: u32,
    page_offset_size: u32) -> u32 {
    let d = max_order;

    let page_size = key_size * (d - 1) +
        (file_offset_size * d) +
        (page_offset_size * d) + header_size;

    return page_size as u32;
}

pub fn test_page_efficiencies() {
    let optimum = calculate_max_order(MAX_PAGE_SIZE, 64, 16, 4);
    let minimum = calculate_max_order(MAX_PAGE_SIZE, 64, 16, 4 + 2);
    let minimum2 = calculate_max_order(MAX_PAGE_SIZE, 64, 16, 4 + 4);
    let maximum = calculate_max_order(MAX_PAGE_SIZE, 64, 16, 8 + 4);
    let maximum2 = calculate_max_order(MAX_PAGE_SIZE, 64, 16, 8 + 2);

    println!("Minimum:");
    calculate_efficiency(minimum, optimum, 4096, 100000000);
    calculate_efficiency(minimum, optimum, 4096, 1000000000);
    calculate_efficiency(minimum, optimum, 4096, 10000000000);

    println!("Minimum2:");
    calculate_efficiency(minimum2, optimum, 4096, 100_000_000);
    calculate_efficiency(minimum2, optimum, 4096, 1_000_000_000);
    calculate_efficiency(minimum2, optimum, 4096, 10_000_000_000);
    calculate_efficiency(minimum2, optimum, 4096, 100_000_000_000);

    println!("Maximum:");
    calculate_efficiency(maximum, optimum, 4096, 100000000);
    calculate_efficiency(maximum, optimum, 4096, 1000000000);
    calculate_efficiency(maximum, optimum, 4096, 10000000000);

    println!("Maximum2:");
    calculate_efficiency(maximum2, optimum, 4096, 100000000);
    calculate_efficiency(maximum2, optimum, 4096, 1000000000);
    calculate_efficiency(maximum2, optimum, 4096, 10000000000);


    let page_size = calculate_page_size(MAX_ORDER, 64, 16, 4, 4);
    let optimum3 = calculate_page_size(MAX_ORDER, 64, 16, 8, 0);

    println!("Page Size: {}", page_size);
    println!("Efficiency:");
    calculate_efficiency(page_size, optimum3, 4096, 100000000);
    calculate_efficiency(page_size, optimum3, 4096, 1000000000);
    calculate_efficiency(page_size, optimum3, 4096, 10000000000);
}