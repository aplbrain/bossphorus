pub mod data_manager {
    /// Data management module.
    ///
    /// Handles data IO from disk, or from other sources.
    ///
    /// In short, this currently handles all of the data IO from cuboids. No
    /// one else should have to worry about slicing and dicing, but if you do
    /// want to, you can use `data_manager::get_cuboids_and_indices`, which is
    /// a lot prettier than my Python implementation, if I do say so myself.
    use crate::intern;

    use intern::remote::BossRemote;
    use ndarray::{s, Array, Array3};
    use std::collections::HashMap;
    use std::fmt;
    use std::fs;
    use std::io::prelude::*;
    use std::path::Path;

    #[derive(Copy, Clone, PartialEq, Eq, Hash)]
    pub struct Vector3 {
        /// A vector of X, Y, and Z members.
        ///
        /// Partially used as a way to learn Rust structs; I imagine there is a
        /// more elegant stdlib way to encode this. But it sure is useful when
        /// you're swapping back and forth between XYZ coordinate ordering and
        /// ZYX C-ordered strides!
        pub x: u64,
        pub y: u64,
        pub z: u64,
    }

    impl fmt::Display for Vector3 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            return write!(f, "x{}_y{}_z{}", self.x, self.y, self.z);
        }
    }

    pub trait DataManager {
        /// A DataManager must be able to get and put data.
        ///
        /// The only exception to this is the NullDataManager, which acts as a
        /// sink for failed requests.
        fn get_data(
            &self,
            uri: String,
            resoluton: u8,
            origin: Vector3,
            destination: Vector3,
        ) -> ndarray::Array3<u8>;
        fn put_data(
            &self,
            uri: String,
            resoluton: u8,
            origin: Vector3,
            data: ndarray::Array3<u8>,
        ) -> bool;

        /// Default to returning a null data manager to catch failed requests.
        fn get_next_layer(&self) -> &dyn DataManager {
            return &NullDataManager {};
        }
    }

    /// A struct placeholder for the NullDataManager.
    pub struct NullDataManager {}

    impl DataManager for NullDataManager {
        fn get_data(
            &self,
            uri: String,
            resoluton: u8,
            origin: Vector3,
            destination: Vector3,
        ) -> ndarray::Array3<u8> {
            // return Array::zeros((
            //     (destination.z - origin.z) as usize,
            //     (destination.y - origin.y) as usize,
            //     (destination.x - origin.x) as usize,
            // ));

            // Alternatively:
            panic!("Failed to get data.")
        }
        fn put_data(
            &self,
            uri: String,
            resoluton: u8,
            origin: Vector3,
            data: ndarray::Array3<u8>,
        ) -> bool {
            panic!("Failed to put data.")
        }
    }

    pub struct ChunkedBloscFileDataManager {
        /// A DataManager. Specifically, a filesystem data manager.
        ///
        /// The closest Python analog of this is the FileSystemStorageManager
        /// which handles cuboid IO from disk. I am absolutely floored at how
        /// much faster this one is than the Python version. They should have
        /// sent a poet.
        file_path: String,
        cuboid_size: Vector3,
        next_layer: Box<dyn DataManager>,
    }

    /// Get a mapping of cuboid indices to the cutout indices within it.
    ///
    /// This sounds a lot more complicated than it actually is, and the
    /// code for it is way more complicated than it feels like it should need
    /// to be.
    ///
    /// In essence, all this does is convert global cutout coordinates into a
    /// bunch of individual cuboids' cutout coordinates. For a large cutout,
    /// most of these will include the entire cuboid (so the value in that kv
    /// pair will be the same as `cuboid_size`). This is the sort of function
    /// that is worth writing once and then never again, so I've written it
    /// here again.
    ///
    /// # Arguments
    ///
    /// * `coords_start` - A vector that indicates the global start position
    /// * `coords_stop` - A vector that indicates the global stop indices
    /// * `cuboid_size` - A vector3 that indicates the XYZ cuboid size on disk
    ///
    /// # Returns
    ///
    /// * Mapping of cuboid ID to cutouts `(Vector3, Vector3)`
    ///
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

    impl ChunkedBloscFileDataManager {
        /// A DataManager handles data IO from disk (and eventually cache).
        ///
        /// Create a new DataManager with a file_path on disk to which cuboids
        /// will be written, and a default cuboid_size (e.g. 512*512*16).
        pub fn new(file_path: String, cuboid_size: Vector3) -> ChunkedBloscFileDataManager {
            return ChunkedBloscFileDataManager {
                file_path,
                cuboid_size,
                next_layer: Box::new(NullDataManager {}),
            };
        }

        pub fn new_with_layer(
            file_path: String,
            cuboid_size: Vector3,
            next_layer: Box<dyn DataManager>,
        ) -> ChunkedBloscFileDataManager {
            return ChunkedBloscFileDataManager {
                file_path,
                cuboid_size,
                next_layer,
            };
        }
    }

    impl DataManager for ChunkedBloscFileDataManager {
        /// TODO: `has_data`
        // fn has_data(&self) -> bool {
        //     return true;
        // }

        /// Get data from a specified cutout region.
        ///
        /// # Arguments
        ///
        /// * `origin` - The start position of the cutout (global coords)
        /// * `destination` - The end position in global coords
        ///
        /// # Returns
        ///
        /// * 3D Array
        ///
        fn get_data(
            &self,
            uri: String,
            res: u8,
            origin: Vector3,
            destination: Vector3,
        ) -> ndarray::Array3<u8> {
            let cuboids = get_cuboids_and_indices(origin, destination, self.cuboid_size);

            let boss_uri: Vec<&str> = uri.split("://").collect();

            let mut large_array: Array3<u8> = Array::zeros((
                (destination.z - origin.z) as usize,
                (destination.y - origin.y) as usize,
                (destination.x - origin.x) as usize,
            ));

            for (cuboid_index, (start_ind, stop_ind)) in &cuboids {
                let filename = format!(
                    "{}/{}/{}/{}",
                    self.file_path, boss_uri[1], res, cuboid_index
                );

                let filepath = Path::new(&filename);

                // Get the coordinates of this cuboid out of the cutout volume:
                let z_start = ((cuboid_index.z * self.cuboid_size.z) + start_ind.z) - origin.z;
                let z_stop = ((cuboid_index.z * self.cuboid_size.z) + stop_ind.z) - origin.z;
                let y_start = ((cuboid_index.y * self.cuboid_size.y) + start_ind.y) - origin.y;
                let y_stop = ((cuboid_index.y * self.cuboid_size.y) + stop_ind.y) - origin.y;
                let x_start = ((cuboid_index.x * self.cuboid_size.x) + start_ind.x) - origin.x;
                let x_stop = ((cuboid_index.x * self.cuboid_size.x) + stop_ind.x) - origin.x;

                let array: Array3<u8>;
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
                    // TODO: This is a cache miss.
                    // Right now, we just pass to the next layer, but we can
                    // certainly be smarter about this.

                    let z_cuboid_start = cuboid_index.z * self.cuboid_size.z;
                    let z_cuboid_stop = (1 + cuboid_index.z) * self.cuboid_size.z;
                    let y_cuboid_start = cuboid_index.y * self.cuboid_size.y;
                    let y_cuboid_stop = (1 + cuboid_index.y) * self.cuboid_size.y;
                    let x_cuboid_start = cuboid_index.x * self.cuboid_size.x;
                    let x_cuboid_stop = (1 + cuboid_index.x) * self.cuboid_size.x;

                    array = self.get_next_layer().get_data(
                        boss_uri[1].to_string(),
                        res,
                        Vector3 {
                            x: x_cuboid_start,
                            y: y_cuboid_start,
                            z: z_cuboid_start,
                        },
                        Vector3 {
                            x: x_cuboid_stop,
                            y: y_cuboid_stop,
                            z: z_cuboid_stop,
                        },
                    );

                    // Put this cuboid into storage for next time:
                    // TODO: We should be abstracting cache management; just
                    //       dumping data back into the datamanager is ugly
                    //       and will be impossible to maintain.
                    self.put_data(
                        uri.clone(),
                        res,
                        Vector3 {
                            x: x_cuboid_start,
                            y: y_cuboid_start,
                            z: z_cuboid_start,
                        },
                        array.clone(),
                    );
                }

                let new_data = array.slice(s![
                    start_ind.z as usize..stop_ind.z as usize,
                    start_ind.y as usize..stop_ind.y as usize,
                    start_ind.x as usize..stop_ind.x as usize
                ]);

                // Insert data cutout into large array
                large_array
                    .slice_mut(s![
                        z_start as usize..z_stop as usize,
                        y_start as usize..y_stop as usize,
                        x_start as usize..x_stop as usize,
                    ])
                    .assign(&new_data);
            }

            return large_array;
        }

        /// Upload data (write to the files).
        ///
        /// # Arguments
        ///
        /// * `data` - The good stuff
        /// * `origin` - The start position of the cutout (global coords)
        /// * `destination` - The end position in global coords
        ///
        /// # Returns
        ///
        /// * Boolean of success
        ///
        fn put_data(
            &self,
            uri: String,
            res: u8,
            origin: Vector3,
            data: ndarray::Array3<u8>,
        ) -> bool {
            let cuboids = get_cuboids_and_indices(
                origin,
                Vector3 {
                    x: origin.x + data.len_of(ndarray::Axis(2)) as u64,
                    y: origin.y + data.len_of(ndarray::Axis(1)) as u64,
                    z: origin.z + data.len_of(ndarray::Axis(0)) as u64,
                },
                self.cuboid_size,
            );
            let boss_uri: Vec<&str> = uri.split("://").collect();

            for (cuboid_index, (start_ind, stop_ind)) in &cuboids {
                let filename = format!(
                    "{}/{}/{}/{}",
                    self.file_path, boss_uri[1], res, cuboid_index
                );

                let filepath = Path::new(&filename);
                let mut array: Array3<u8>;
                // Get existing data:
                if filepath.exists() && fs::read(&filepath).unwrap().len() > 0 {
                    let read_data = fs::read(&filepath).unwrap();
                    array = Array::from_shape_vec(
                        (
                            self.cuboid_size.z as usize,
                            self.cuboid_size.y as usize,
                            self.cuboid_size.x as usize,
                        ),
                        read_data,
                    )
                    .unwrap();
                } else {
                    let dir_path: Vec<&str> = filename.split("/").collect();
                    let dir_path_str = dir_path[..dir_path.len() - 1].join("/");
                    match fs::create_dir_all(&dir_path_str) {
                        Ok(a) => a,
                        _ => unreachable!(), // Failed to create file somehow...
                    };
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

                // Get the coordinates of this cuboid out of the cutout volume:
                let z_start = ((cuboid_index.z * self.cuboid_size.z) + start_ind.z) - origin.z;
                let z_stop = ((cuboid_index.z * self.cuboid_size.z) + stop_ind.z) - origin.z;
                let y_start = ((cuboid_index.y * self.cuboid_size.y) + start_ind.y) - origin.y;
                let y_stop = ((cuboid_index.y * self.cuboid_size.y) + stop_ind.y) - origin.y;
                let x_start = ((cuboid_index.x * self.cuboid_size.x) + start_ind.x) - origin.x;
                let x_stop = ((cuboid_index.x * self.cuboid_size.x) + stop_ind.x) - origin.x;

                // Write cuboid to the array:
                array
                    .slice_mut(s![
                        start_ind.z as usize..stop_ind.z as usize,
                        start_ind.y as usize..stop_ind.y as usize,
                        start_ind.x as usize..stop_ind.x as usize
                    ])
                    .assign(&data.slice(s![
                        z_start as usize..z_stop as usize,
                        y_start as usize..y_stop as usize,
                        x_start as usize..x_stop as usize,
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

        fn get_next_layer(&self) -> &dyn DataManager {
            return self.next_layer.as_ref();
        }
    }

    pub struct BossDBRelayDataManager {
        /// The BossDBRelayDataManager accepts requests for data and relays it
        /// to a BossDB API using `intern-rust`.
        ///
        /// It currently relays the general-access token `public` instead of
        /// the user's token, which is arguably Not The Correct Thing To Do but
        /// certainly is way easier to implement.
        /// Why is this hard? You don't know which user requested data to load
        /// it into the cache in the first place. So if you then receive a
        /// subsequent request, there's no guarantee that the new user actually
        /// has permission to see that dataset.
        /// In order to avoid this altogether, and to avoid permissions getting
        /// out of sync with the upstream BossDB, better to just let this be
        /// public-only. (If you want to change this behavior, you can always
        /// change the token to that of a BossDB administrator, but...
        /// obviously, watch out.)
        token: String,
        host: String,
        protocol: String,
    }

    impl BossDBRelayDataManager {
        /// A BossDBRelayDataManager handles data transactions with a BossDB
        /// API (https://bossdb.org).
        ///
        /// # Arguments
        ///
        /// * `protocol` - Generally one of `http` or `https`
        /// * `host` - The API root of the BossDB instance (e.g. `api.bossdb.io`)
        /// * `token` - The token to use for ALL requests from this mgr
        ///
        pub fn new(protocol: String, host: String, token: String) -> BossDBRelayDataManager {
            println!("host: {}", host);
            BossDBRelayDataManager {
                protocol,
                host,
                token,
            }
        }
    }

    impl DataManager for BossDBRelayDataManager {
        /// Get data from the upstream BossDB.
        fn get_data(
            &self,
            uri: String,
            res: u8,
            origin: Vector3,
            destination: Vector3,
        ) -> ndarray::Array3<u8> {
            let remote = BossRemote::new(
                self.protocol.to_string(),
                self.host.to_string(),
                self.token.to_string(),
            );

            let data = remote
                .get_cutout(
                    format!("bossdb://{}", uri),
                    res,
                    (origin.x, destination.x),
                    (origin.y, destination.y),
                    (origin.z, destination.z),
                )
                .unwrap();
            return data;
        }

        /// Unimplemented. Don't do this, I think.
        fn put_data(
            &self,
            _uri: String,
            _resoluton: u8,
            _origin: Vector3,
            _data: ndarray::Array3<u8>,
        ) -> bool {
            panic!("Putting data with the BossDB relay is currently not supported.")
        }
    }
}
