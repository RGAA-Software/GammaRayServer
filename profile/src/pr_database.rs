use std::sync::Arc;
use futures_util::StreamExt;
use mongodb::{
    bson::{Document, doc},
    Client,
    Collection
};
use mongodb::options::FindOptions;
use crate::pr_device::PrDevice;

pub struct PrDatabase {
    pub client: Option<Arc<tokio::sync::Mutex<Client>>>,
    pub c_device: Option<Arc<tokio::sync::Mutex<Collection<PrDevice>>>>,
}

impl PrDatabase {
    pub fn new() -> std::sync::Arc<tokio::sync::Mutex<PrDatabase>> {
        std::sync::Arc::new(tokio::sync::Mutex::new(PrDatabase{
            client: None,
            c_device: None,
        }))
    }

    pub async fn init(&mut self) -> bool {
        // Replace the placeholder with your Atlas connection string
        let uri = "mongodb://localhost:27017/";
        // Create a new client and connect to the server
        let client = Client::with_uri_str(uri).await;
        if let Err(e) = client {
            println!("error connecting to MongoDB: {}", e);
            return false;
        }
        let client = client.unwrap();
        // Get a handle on the movies collection
        let database = client.database("db_profile");
        let c_device: Collection<PrDevice> = database.collection("c_device");
        self.c_device = Some(Arc::new(tokio::sync::Mutex::new(c_device)));
        
        return true;
    }
    
    pub async fn query_devices(&mut self, page: i32, page_size: i32) -> Vec<PrDevice> {
        let c_device = self.c_device.clone().unwrap().clone();
        let filter = doc! { };
        let mut cursor = c_device.lock().await
            .find(filter)
            //.sort(doc! { "title": 1 })
            .await
            .unwrap();

        let mut devices: Vec<PrDevice> = Vec::new();
        while let Some(device) = cursor.next().await {
            if let Ok(device) = device {
                devices.push(device);
            }
        }
        devices
    }
    
    pub async fn insert_device(&mut self, device: PrDevice) -> bool {
        let c_device = self.c_device.clone().unwrap().clone();
        let r = c_device.lock().await.insert_one(device).await;
        if let Err(e) = r {
            println!("error inserting device: {}", e);
            return false;
        }
        true
    }
}