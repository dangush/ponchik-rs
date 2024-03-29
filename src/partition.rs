use rand::{prelude::StdRng, seq::SliceRandom, SeedableRng};
use tracing::instrument;

#[instrument]
pub fn random_partition<T: Clone + std::fmt::Debug>(xs: &mut [T], size: usize) -> Vec<Vec<T>> {
    if xs.is_empty() || size < 1 {
        return vec![xs.into()];
    }

    let mut rng = StdRng::from_entropy();
    xs.shuffle(&mut rng);

    let mut partitions: Vec<_> = xs.chunks(size).map(ToOwned::to_owned).collect();
    if partitions.len() > 1 && partitions.last().unwrap().len() != size {
        // Last partition has less elements than expected
        let _last_partition = partitions.remove(partitions.len() - 1);

        // let last_idx = partitions.len() - 1;
        // partitions[last_idx].append(&mut last_partition);
    }

    // TODO: Loop over partitions and remove any duplicates. 
    // How the fuck am I going to handle the replacement of these duplicates with just .chunks()? 
    partitions
}
