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
use tokio::sync::Mutex;
use crate::dash_element::DashElement;
use crate::dash_group::DashGroup;

pub struct DashDatabase {
    pub client: Option<Arc<Mutex<Client>>>,
    pub c_group: Option<Arc<Mutex<Collection<DashGroup>>>>,
    pub c_element: Option<Arc<Mutex<Collection<DashElement>>>>,
}

impl DashDatabase {
    pub fn new() -> Self {
        DashDatabase{
            client: None,
            c_group: None,
            c_element: None,
        }
    }

    pub async fn init(&mut self) -> bool {
        let uri = "mongodb://localhost:27017/";
        // Create a new client and connect to the server
        let client = Client::with_uri_str(uri).await;
        if let Err(e) = client {
            println!("error connecting to MongoDB: {}", e);
            return false;
        }
        let client = client.unwrap();
        // Get a handle on the movies collection
        let database = client.database("db_dashboard");
        // group
        let c_group: Collection<DashGroup> = database.collection("c_group");
        self.c_group = Some(Arc::new(Mutex::new(c_group)));

        // element
        let c_element: Collection<DashElement> = database.collection("c_element");
        self.c_element = Some(Arc::new(Mutex::new(c_element)));

        true
    }
    
    pub async fn query_groups(&mut self, page: i32, page_size: i32) -> Vec<DashGroup> {
        let c_device = self.c_group.clone().unwrap().clone();
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

        let mut groups: Vec<DashGroup> = Vec::new();
        while let Some(group) = cursor.next().await {
            if let Err(e) = group {
                println!("error connecting to MongoDB: {}", e);
            }else {
                println!("device: {:?}", group);
                groups.push(group.unwrap());
            }
        }
        groups
    }
    
    pub async fn insert_device(&mut self, group: DashGroup) -> bool {
        let c_group = self.c_group.clone().unwrap().clone();
        let r = c_group.lock().await.insert_one(group).await;
        if let Err(e) = r {
            println!("error inserting device: {}", e);
            return false;
        }
        true
    }
    
    pub async fn find_device_by_id(&self, device_id: String) -> Option<DashGroup> {
        let c_device = self.c_group.clone().unwrap().clone();
        let filter = doc! { 
            "device_id": device_id,
        };
        let r = c_device.lock().await.find_one(filter, ).await;
        r.unwrap_or(None)
    }

    pub async fn update_device(&mut self, device_id: String, update_info: HashMap<String, String>) -> bool {
        let c_device = self.c_group.clone().unwrap().clone();
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
        let c_device = self.c_group.clone().unwrap().clone();
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