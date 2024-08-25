// use tokio::sync::Semaphore;
// use std::sync::Arc;
// use tokio::task;
//
// #[tokio::main]
// async fn main() {
//     let semaphore = Arc::new(Semaphore::new(3)); // Semaphore with 3 permits
//     let mut handles = vec![];
//
//     for _ in 0..5 {
//         let permit = Arc::clone(&semaphore);
//         let handle = task::spawn(async move {
//             let _permit = permit.acquire().await.unwrap(); // Acquiring a permit
//             println!("Acquired permit");
//             // Critical section
//         });
//         handles.push(handle);
//     }
//
//     for handle in handles {
//         handle.await.unwrap();
//     }
// }