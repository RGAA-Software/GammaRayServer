use std::collections::HashMap;
use std::sync::Arc;
use futures_util::StreamExt;
use mongodb::{
    bson::{Document, doc},
    Client,
    Collection
};
use mongodb::bson::Bson;
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
        let skip = (page-1) * page_size;
        let limit = page_size as i64;
        let mut cursor = c_device.lock().await
            .find(filter)
            // .skip(skip as u64)
            // .limit(limit)
            .await
            .unwrap();

        println!("query device, skip:{} - limit:{}", skip, limit);

        let mut devices: Vec<PrDevice> = Vec::new();
        while let Some(device) = cursor.next().await {
            if let Err(e) = device {
                println!("error connecting to MongoDB: {}", e);
            }else {
                println!("device: {:?}", device);
                devices.push(device.unwrap());
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
    
    pub async fn find_device_by_id_and_seed(&self, device_id: String, seed: String) -> Option<PrDevice> {
        let c_device = self.c_device.clone().unwrap().clone();
        let filter = doc! { 
            "device_id": device_id,
            "seed": seed,
        };
        let r = c_device.lock().await.find_one(filter, ).await;
        r.unwrap_or(None)
    }
    
    pub async fn find_device_by_id(&self, device_id: String) -> Option<PrDevice> {
        let c_device = self.c_device.clone().unwrap().clone();
        let filter = doc! { 
            "device_id": device_id,
        };
        let r = c_device.lock().await.find_one(filter, ).await;
        r.unwrap_or(None)
    }

    pub async fn update_device(&mut self, device_id: String, update_info: HashMap<String, String>) -> bool {
        let c_device = self.c_device.clone().unwrap().clone();
        let filter_doc = doc! {
            "device_id": device_id,
        };
        let mut update_doc = doc! {};
        let mut sub_update_doc = doc! {};
        for (k, v) in update_info {
            sub_update_doc.insert(k, v);
        }
        sub_update_doc.insert("last_update_timestamp", base::get_current_timestamp());
        update_doc.insert("$set", sub_update_doc);
        let r = c_device.lock().await.update_one(filter_doc, update_doc).await;
        if let Err(e) = r {
            println!("error updating device: {}", e);
            false
        }
        else {
            true
        }
    }

    pub async fn update_device_field<T>(&mut self, device_id: String, key: String, val: T) -> bool where T: Into<Bson> {
        let c_device = self.c_device.clone().unwrap().clone();
        let filter_doc = doc! {
            "device_id": device_id,
        };
        let mut update_doc = doc! {};
        let mut sub_update_doc = doc! {
            key: val,
        };

        sub_update_doc.insert("last_update_timestamp", base::get_current_timestamp());
        update_doc.insert("$set", sub_update_doc);
        let r = c_device.lock().await.update_one(filter_doc, update_doc).await;
        if let Err(e) = r {
            println!("error updating device: {}", e);
            false
        }
        else {
            true
        }
    }
}