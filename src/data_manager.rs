pub mod data_manager {

    use crate::intern;

    use ndarray::{s, Array, Array3};
    use std::collections::HashMap;
    use std::fmt;
    use std::fs;
    // use std::fs::File;
    use std::io::prelude::*;
    use std::path::Path;

    #[derive(Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Vector3 {
        pub x: u64,
        pub y: u64,
        pub z: u64,
    }

    impl fmt::Display for Vector3 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            return write!(f, "x{}_y{}_z{}", self.x, self.y, self.z);
        }
    }

    pub struct DataManager {
        file_path: String,
        cuboid_size: Vector3,
    }

    pub fn get_cuboids_and_indices(
        coords_start: Vector3,
        coords_stop: Vector3,
        cuboid_size: Vector3,
    ) -> HashMap<Vector3, (Vector3, Vector3)> {
        let mut cuboids = HashMap::new();

        let start_cuboid = Vector3 {
            x: coords_start.x / cuboid_size.x,
            y: coords_start.y / cuboid_size.y,
            z: coords_start.z / cuboid_size.z,
        };

        let stop_cuboid = Vector3 {
            x: coords_stop.x / cuboid_size.x,
            y: coords_stop.y / cuboid_size.y,
            z: coords_stop.z / cuboid_size.z,
        };

        for cuboid_index_x in start_cuboid.x..=stop_cuboid.x {
            for cuboid_index_y in start_cuboid.y..=stop_cuboid.y {
                for cuboid_index_z in start_cuboid.z..=stop_cuboid.z {
                    // if overflow { cuboid_size } else { size % cuboid_size }

                    // TODO: This entire block is hideous.
                    let start_coords = Vector3 {
                        x: if coords_start.x <= cuboid_size.x * cuboid_index_x {
                            0
                        } else {
                            coords_start.x % cuboid_size.x
                        },
                        y: if coords_start.y <= cuboid_size.y * cuboid_index_y {
                            0
                        } else {
                            coords_start.y % cuboid_size.y
                        },
                        z: if coords_start.z <= cuboid_size.z * cuboid_index_z {
                            0
                        } else {
                            coords_start.z % cuboid_size.z
                        },
                    };

                    let stop_coords = Vector3 {
                        x: if coords_stop.x >= cuboid_size.x * (1 + cuboid_index_x) {
                            cuboid_size.x
                        } else {
                            coords_stop.x % cuboid_size.x
                        },
                        y: if coords_stop.y >= cuboid_size.y * (1 + cuboid_index_y) {
                            cuboid_size.y
                        } else {
                            coords_stop.y % cuboid_size.y
                        },
                        z: if coords_stop.z >= cuboid_size.z * (1 + cuboid_index_z) {
                            cuboid_size.z
                        } else {
                            coords_stop.z % cuboid_size.z
                        },
                    };

                    cuboids.insert(
                        Vector3 {
                            x: cuboid_index_x,
                            y: cuboid_index_y,
                            z: cuboid_index_z,
                        },
                        (start_coords, stop_coords),
                    );
                }
            }
        }

        return cuboids;
    }

    impl DataManager {
        pub fn new(file_path: String, cuboid_size: Vector3) -> DataManager {
            return DataManager {
                file_path,
                cuboid_size,
            };
        }

        // fn has_data(&self) -> bool {
        //     return true;
        // }

        pub fn get_data(&self, origin: Vector3, destination: Vector3) -> ndarray::Array3<u8> {
            let cuboids = get_cuboids_and_indices(origin, destination, self.cuboid_size);

            let mut large_array: Array3<u8> = Array::zeros((
                (destination.z - origin.z) as usize,
                (destination.y - origin.y) as usize,
                (destination.x - origin.x) as usize,
            ));

            for (cuboid_index, (start_ind, stop_ind)) in &cuboids {
                let filename = format!("{}/{}", self.file_path, cuboid_index);

                let filepath = Path::new(&filename);
                let mut array: Array3<u8>;
                // Get existing data:
                if filepath.exists() {
                    let data = fs::read(&filename).unwrap();
                    array = Array::from_shape_vec(
                        (
                            self.cuboid_size.z as usize,
                            self.cuboid_size.y as usize,
                            self.cuboid_size.x as usize,
                        ),
                        data,
                    )
                    .unwrap();
                } else {
                    // TODO: download data from upstream. This is a cache miss!
                    array = Array::zeros((
                        self.cuboid_size.z as usize,
                        self.cuboid_size.y as usize,
                        self.cuboid_size.x as usize,
                    ));
                }

                // Data cutouts
                // Get the coordinates of this cuboid out of the full volume:
                let z_start = (cuboid_index.z * self.cuboid_size.z) + start_ind.z;
                let z_stop = (cuboid_index.z * self.cuboid_size.z) + stop_ind.z;
                let y_start = (cuboid_index.y * self.cuboid_size.y) + start_ind.y;
                let y_stop = (cuboid_index.y * self.cuboid_size.y) + stop_ind.y;
                let x_start = (cuboid_index.x * self.cuboid_size.x) + start_ind.x;
                let x_stop = (cuboid_index.x * self.cuboid_size.x) + stop_ind.x;

                large_array
                    .slice_mut(s![
                        z_start as isize..z_stop as isize,
                        y_start as isize..y_stop as isize,
                        x_start as isize..x_stop as isize,
                    ])
                    .assign(&array.slice_mut(s![
                        start_ind.z as isize..stop_ind.z as isize,
                        start_ind.y as isize..stop_ind.y as isize,
                        start_ind.x as isize..stop_ind.x as isize
                    ]))
            }

            return large_array;
        }

        pub fn put_data(&self, data: ndarray::Array3<u8>, origin: Vector3) -> bool {
            let cuboids = get_cuboids_and_indices(
                origin,
                Vector3 {
                    x: origin.x + data.len_of(ndarray::Axis(2)) as u64,
                    y: origin.y + data.len_of(ndarray::Axis(1)) as u64,
                    z: origin.z + data.len_of(ndarray::Axis(0)) as u64,
                },
                self.cuboid_size,
            );

            for (cuboid_index, (start_ind, stop_ind)) in &cuboids {
                let filename = format!("{}/{}", self.file_path, cuboid_index);

                let filepath = Path::new(&filename);
                let mut array: Array3<u8>;
                // Get existing data:
                if filepath.exists() {
                    let data = fs::read(&filename).unwrap();
                    array = Array::from_shape_vec(
                        (
                            self.cuboid_size.z as usize,
                            self.cuboid_size.y as usize,
                            self.cuboid_size.x as usize,
                        ),
                        data,
                    )
                    .unwrap();
                } else {
                    match fs::File::create(&filepath) {
                        Err(why) => panic!(
                            "couldn't create {}: {}",
                            filepath.display(),
                            why.to_string()
                        ),
                        Ok(file) => file,
                    };
                    array = Array::zeros((
                        self.cuboid_size.z as usize,
                        self.cuboid_size.y as usize,
                        self.cuboid_size.x as usize,
                    ));
                }

                // Data cutouts
                // Get the coordinates of this cuboid out of the full volume:
                let z_start = (cuboid_index.z * self.cuboid_size.z) + start_ind.z;
                let z_stop = (cuboid_index.z * self.cuboid_size.z) + stop_ind.z;
                let y_start = (cuboid_index.y * self.cuboid_size.y) + start_ind.y;
                let y_stop = (cuboid_index.y * self.cuboid_size.y) + stop_ind.y;
                let x_start = (cuboid_index.x * self.cuboid_size.x) + start_ind.x;
                let x_stop = (cuboid_index.x * self.cuboid_size.x) + stop_ind.x;

                // Write cuboid to the array:
                array
                    .slice_mut(s![
                        start_ind.z as isize..stop_ind.z as isize,
                        start_ind.y as isize..stop_ind.y as isize,
                        start_ind.x as isize..stop_ind.x as isize
                    ])
                    .assign(&data.slice(s![
                        z_start as isize..z_stop as isize,
                        y_start as isize..y_stop as isize,
                        x_start as isize..x_stop as isize,
                    ]));

                // Write cuboid to disk:
                let mut file = fs::File::create(&filepath).unwrap();
                match file.write_all(&array.into_raw_vec()) {
                    Err(why) => println!(
                        "Failed to write cuboid {}: {}",
                        cuboid_index,
                        why.to_string()
                    ),
                    Ok(_) => (),
                }
            }
            return true;
        }
    }
}
