use std::collections::HashMap;
use std::iter::FromIterator;

use criterion::{Criterion, criterion_group, criterion_main};

use rand::{distributions::Alphanumeric, Rng, SeedableRng};
use tempfile::TempDir;

use kvs::{KvsEngine, KvStore, SledKvsEngine};

fn get_random_write_data() -> HashMap<String, String> {
    let mut key_val: HashMap<String, String> = HashMap::new();
    let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
    let rng_ref = &mut rng;
    for _i in 0..100 {
        let key_len = rng_ref.gen_range(0..=10000);
        let val_len = rng_ref.gen_range(0..=10000);
        let key: String = rng_ref.sample_iter(&Alphanumeric)
            .take(key_len)
            .map(char::from)
            .collect();
        let val: String = rng_ref.sample_iter(&Alphanumeric)
            .take(val_len)
            .map(char::from)
            .collect();
        key_val.insert(key, val);
    }
    key_val
}

fn get_random_read_keys(data: &HashMap<String, String>) -> Vec<String> {
    let mut rng = rand::rngs::StdRng::from_seed([0; 32]);
    let keys = Vec::from_iter(data.keys());
    let mut result: Vec<String> = vec![];
    for _i in 0..1000 {
        let random_index = rng.gen_range(0..100);
        result.push(keys[random_index].to_string());
    }
    result
}

fn engine_benchmark(c: &mut Criterion) {
    let random_write_data = get_random_write_data();
    let random_read_keys = get_random_read_keys(&random_write_data);
    let kvs_temp_dir = TempDir::new().expect("unable to create temporary working directory");
    let sled_temp_dir = TempDir::new().expect("unable to create temporary working directory");

    c.bench_function("kvs write", |b| {
        b.iter(|| {
            let mut store = KvStore::open(kvs_temp_dir.path()).expect("unable to open db");
            for key_val in random_write_data.iter() {
                store.set(key_val.0.to_string(), key_val.1.to_string()).unwrap();
            }
        });
    });

    c.bench_function("sled write", |b| {
        b.iter(|| {
            let mut store = SledKvsEngine::open(sled_temp_dir.path()).expect("unable to open db");
            for key_val in random_write_data.iter() {
                store.set(key_val.0.to_string(), key_val.1.to_string()).unwrap();
            }
        });
    });

    c.bench_function("kvs read", |b| {
        b.iter(|| {
            let store = KvStore::open(kvs_temp_dir.path()).expect("unable to open db");
            for key in random_read_keys.iter() {
                let val = store.get(key.to_string()).unwrap().unwrap();
                assert_eq!(val, random_write_data.get(key).unwrap().to_string());
            }
        });
    });

    c.bench_function("sled read", |b| {
        b.iter(|| {
            let store = SledKvsEngine::open(sled_temp_dir.path()).expect("unable to open db");
            for key in random_read_keys.iter() {
                let val = store.get(key.to_string()).unwrap().unwrap();
                assert_eq!(val, random_write_data.get(key).unwrap().to_string());
            }
        });
    });
}

criterion_group!(benches, engine_benchmark);
criterion_main!(benches);
