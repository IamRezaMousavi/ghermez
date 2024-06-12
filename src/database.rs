#![allow(non_snake_case)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use pyo3::prelude::*;
use regex::Regex;
use rusqlite::Connection;

use crate::useful_tools::determineConfigFolder;

// This class manages TempDB
// TempDB contains gid of active downloads in every session.
#[pyclass]
pub struct TempDB {
    connection: Arc<Mutex<Connection>>,
}

#[pymethods]
impl TempDB {
    #[new]
    fn new() -> Self {
        // temp_db saves in RAM
        Self {
            connection: Arc::new(Mutex::new(Connection::open_in_memory().unwrap())),
        }
    }

    // temp_db_table contains gid of active downloads.

    fn createTables(&self) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();
        transaction
            .execute(
                "
            CREATE TABLE IF NOT EXISTS single_db_table (
                ID INTEGER,
                gid TEXT PRIMARY KEY,
                status TEXT,
                shutdown TEXT
            )",
                (),
            )
            .unwrap();
        transaction
            .execute(
                "
            CREATE TABLE IF NOT EXISTS queue_db_table (
                ID INTEGER,
                category TEXT PRIMARY KEY,
                shutdown TEXT
            )",
                (),
            )
            .unwrap();
        transaction.commit().unwrap();
    }

    // insert new item in single_db_table
    fn insertInSingleTable(&self, gid: &str) {
        // lock data base
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "
            INSERT INTO single_db_table VALUES (
                NULL,
                ?1,
                'active',
                NULL
            )",
                [gid],
            )
            .unwrap();
    }

    // insert new item in queue_db_table
    fn insertInQueueTable(&self, category: &str) {
        // lock data base
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "
            INSERT INTO queue_db_table VALUES (
                NULL,
                ?1,
                NULL
            )",
                [category],
            )
            .unwrap();
    }

    // this method updates single_db_table
    fn updateSingleTable(&self, dict: HashMap<&str, &str>) {
        // lock data base
        let connection = self.connection.lock().unwrap();

        // update data base if value for the keys is not None
        connection
            .execute(
                "
                UPDATE single_db_table SET
                shutdown = coalesce(?1, shutdown),
                status = coalesce(?2, status)
                WHERE gid = ?3
                ",
                [dict.get(&"shutdown"), dict.get(&"status"), dict.get(&"gid")],
            )
            .unwrap();
    }

    // this method updates queue_db_table
    fn updateQueueTable(&self, dict: HashMap<&str, &str>) {
        // lock data base
        let connection = self.connection.lock().unwrap();

        // update data base if value for the keys is not None
        connection
            .execute(
                "
                UPDATE queue_db_table SET
                shutdown = coalesce(?1, shutdown)
                WHERE category = ?2
                ",
                [dict.get(&"shutdown"), dict.get(&"category")],
            )
            .unwrap();
    }

    // this method returns gid of active downloads
    fn returnActiveGids(&self) -> Vec<String> {
        // lock data base
        let connection = self.connection.lock().unwrap();
        let mut stmt = connection
            .prepare(
                "
        SELECT gid FROM single_db_table WHERE status = 'active'
        ",
            )
            .unwrap();

        let mut gid_list = vec![];

        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            gid_list.push(row.get(0).unwrap());
        }
        gid_list
    }

    // this method returns shutdown value for specific gid
    fn returnGid(&self, gid: &str) -> Option<HashMap<String, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();
        let mut stmt = connection
            .prepare(
                "
                SELECT shutdown, status FROM single_db_table WHERE gid = ?1
                ",
            )
            .unwrap();

        let mut rows = stmt.query([gid]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            return Some(HashMap::from([
                (
                    "shutdown".to_string(),
                    row.get(0).unwrap_or("NULL".to_string()),
                ),
                ("status".to_string(), row.get(1).unwrap()),
            ]));
        }
        None
    }

    // This method returns values of columns for specific category
    fn returnCategory(&self, category: &str) -> Option<HashMap<String, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();
        let mut stmt = connection
            .prepare(
                "
                SELECT shutdown FROM queue_db_table WHERE category = ?1
                ",
            )
            .unwrap();

        let mut rows = stmt.query([category]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            return Some(HashMap::from([(
                "shutdown".to_string(),
                row.get(0).unwrap(),
            )]));
        }
        None
    }

    fn resetDataBase(&self) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        // delete all items
        transaction
            .execute("DELETE FROM single_db_table", ())
            .unwrap();
        transaction
            .execute("DELETE FROM queue_db_table", ())
            .unwrap();
        transaction.commit().unwrap();
    }
}

// plugins.db is store links, when browser plugins are send new links.
// This class is managing plugin.db
#[pyclass]
pub struct PluginsDB {
    connection: Arc<Mutex<Connection>>,
}

#[pymethods]
impl PluginsDB {
    #[new]
    fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(
                Connection::open(determineConfigFolder().join("plugins.db")).unwrap(),
            )),
        }
    }

    // plugins_db_table contains links that sends by browser plugins.

    fn createTables(&self) {
        // lock data base
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "
            CREATE TABLE IF NOT EXISTS plugins_db_table(
                ID INTEGER PRIMARY KEY,
                link TEXT,
                referer TEXT,
                load_cookies TEXT,
                user_agent TEXT,
                header TEXT,
                out TEXT,
                status TEXT
                )
            ",
                (),
            )
            .unwrap();
    }

    // insert new items in plugins_db_table
    fn insertInPluginsTable(&self, list: Vec<HashMap<&str, &str>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let mut transaction = connection.transaction().unwrap();

        let transaction_size = 5;
        for (i, dict) in list.into_iter().enumerate() {
            if i % transaction_size == 0 {
                transaction.commit().unwrap();
                transaction = connection.transaction().unwrap();
            }
            transaction
                .execute(
                    "
                    INSERT INTO plugins_db_table VALUES(
                        NULL, ?1, ?2, ?3, ?4, ?5, ?6, 'new'
                    )
                ",
                    [
                        dict.get("link"),
                        dict.get("referer"),
                        dict.get("load_cookies"),
                        dict.get("user_agent"),
                        dict.get("header"),
                        dict.get("out"),
                    ],
                )
                .unwrap();
        }
        transaction.commit().unwrap();
    }

    fn returnNewLinks(&self) -> Vec<HashMap<String, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();
        let mut stmt = connection
            .prepare(
                "
                SELECT link, referer, load_cookies, user_agent, header, out
                FROM plugins_db_table WHERE status = 'new'
            ",
            )
            .unwrap();

        // chang all rows status to 'old'
        connection
            .execute(
                "
            UPDATE plugins_db_table SET
            status = 'old'
            WHERE status = 'new'
            ",
                (),
            )
            .unwrap();

        let mut new_list = vec![];

        // put the information in tuples in dictionary format and add it to new_list
        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            new_list.push(HashMap::from([
                ("link".to_string(), row.get(0).unwrap()),
                ("referer".to_string(), row.get(1).unwrap()),
                ("load_cookies".to_string(), row.get(2).unwrap()),
                ("user_agent".to_string(), row.get(3).unwrap()),
                ("header".to_string(), row.get(4).unwrap()),
                ("out".to_string(), row.get(5).unwrap()),
            ]));
        }

        // return results in list format!
        // every member of this list is a dictionary.
        // every dictionary contains download information
        new_list
    }

    // delete old links from data base
    fn deleteOldLinks(&self) {
        // lock data base
        let connection = self.connection.lock().unwrap();

        connection
            .execute("DELETE FROM plugins_db_table WHERE status = 'old'", ())
            .unwrap();
    }
}

// ghermez main data base contains downloads information
// This class is managing ghermez.db
#[pyclass]
pub struct DataBase {
    connection: Arc<Mutex<Connection>>,
}

#[pymethods]
impl DataBase {
    #[new]
    fn new() -> Self {
        let connection = Arc::new(Mutex::new(
            Connection::open(determineConfigFolder().join("ghermez.db")).unwrap(),
        ));

        let cnn = connection.lock().unwrap();

        // To debuging
        // cnn.trace(Some(|s| {
        //     println!("{s}");
        // }));

        // turn FOREIGN KEY Support on!
        cnn.execute("PRAGMA foreign_keys = ON", ()).unwrap();
        drop(cnn);

        Self { connection }
    }

    // queues_list contains name of categories and category settings
    fn createTables(&self) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        // Create category_db_table and add 'All Downloads' and 'Single Downloads' to it
        transaction
            .execute(
                "
                CREATE TABLE IF NOT EXISTS category_db_table(
                    category TEXT PRIMARY KEY,
                    start_time_enable TEXT,
                    start_time TEXT,
                    end_time_enable TEXT,
                    end_time TEXT,
                    reverse TEXT,
                    limit_enable TEXT,
                    limit_value TEXT,
                    after_download TEXT,
                    gid_list TEXT
                )",
                (),
            )
            .unwrap();

        // download table contains download table download items information
        transaction
            .execute(
                "
                CREATE TABLE IF NOT EXISTS download_db_table(
                    file_name TEXT,
                    status TEXT,
                    size TEXT,
                    downloaded_size TEXT,
                    percent TEXT,
                    connections TEXT,
                    rate TEXT,
                    estimate_time_left TEXT,
                    gid TEXT PRIMARY KEY,
                    link TEXT,
                    first_try_date TEXT,
                    last_try_date TEXT,
                    category TEXT,
                    FOREIGN KEY(category) REFERENCES category_db_table(category)
                    ON UPDATE CASCADE
                    ON DELETE CASCADE
                )",
                (),
            )
            .unwrap();

        // addlink_db_table contains addlink window download information
        transaction
            .execute(
                "
            CREATE TABLE IF NOT EXISTS addlink_db_table(
                ID INTEGER PRIMARY KEY,
                gid TEXT,
                out TEXT,
                start_time TEXT,
                end_time TEXT,
                link TEXT,
                ip TEXT,
                port TEXT,
                proxy_user TEXT,
                proxy_passwd TEXT,
                download_user TEXT,
                download_passwd TEXT,
                connections TEXT,
                limit_value TEXT,
                download_path TEXT,
                referer TEXT,
                load_cookies TEXT,
                user_agent TEXT,
                header TEXT,
                after_download TEXT,
                FOREIGN KEY(gid) REFERENCES download_db_table(gid)
                ON UPDATE CASCADE
                ON DELETE CASCADE
            )
            ",
                (),
            )
            .unwrap();

        // video_finder_db_table contains addlink window download information
        transaction
            .execute(
                "
            CREATE TABLE IF NOT EXISTS video_finder_db_table(
                ID INTEGER PRIMARY KEY,
                video_gid TEXT,
                audio_gid TEXT,
                video_completed TEXT,
                audio_completed TEXT,
                muxing_status TEXT,
                checking TEXT,
                download_path TEXT,
                FOREIGN KEY(video_gid) REFERENCES download_db_table(gid)
                ON DELETE CASCADE,
                FOREIGN KEY(audio_gid) REFERENCES download_db_table(gid)
                ON DELETE CASCADE
            )
            ",
                (),
            )
            .unwrap();
        transaction.commit().unwrap();

        // job is done! open the lock
        drop(connection);

        // add 'All Downloads' and 'Single Downloads' to the category_db_table if they wasn't added.
        let answer = self.searchCategoryInCategoryTable("All Downloads");
        if answer.is_none() {
            let all_downloads_dict = HashMap::from([
                ("category", "All Downloads"),
                ("start_time_enable", "no"),
                ("start_time", "0:0"),
                ("end_time_enable", "no"),
                ("end_time", "0:0"),
                ("reverse", "no"),
                ("limit_enable", "no"),
                ("limit_value", "OK"),
                ("after_download", "no"),
                ("gid_list", "[]"),
            ]);
            let single_downloads_dict = HashMap::from([
                ("category", "Single Downloads"),
                ("start_time_enable", "no"),
                ("start_time", "0:0"),
                ("end_time_enable", "no"),
                ("end_time", "0:0"),
                ("reverse", "no"),
                ("limit_enable", "no"),
                ("limit_value", "OK"),
                ("after_download", "no"),
                ("gid_list", "[]"),
            ]);
            self.insertInCategoryTable(all_downloads_dict);
            self.insertInCategoryTable(single_downloads_dict);
        }

        // add default queue with the name 'Scheduled Downloads'
        let answer = self.searchCategoryInCategoryTable("Scheduled Downloads");
        if answer.is_none() {
            let scheduled_downloads_dict = HashMap::from([
                ("category", "Scheduled Downloads"),
                ("start_time_enable", "no"),
                ("start_time", "0:0"),
                ("end_time_enable", "no"),
                ("end_time", "0:0"),
                ("reverse", "no"),
                ("limit_enable", "no"),
                ("limit_value", "OK"),
                ("after_download", "no"),
                ("gid_list", "[]"),
            ]);
            self.insertInCategoryTable(scheduled_downloads_dict);
        }
    }

    // insert new category in category_db_table
    fn insertInCategoryTable(&self, dict: HashMap<&str, &str>) {
        // lock data base
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "
            INSERT INTO category_db_table VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10
            )
            ",
                [
                    dict.get("category"),
                    dict.get("start_time_enable"),
                    dict.get("start_time"),
                    dict.get("end_time_enable"),
                    dict.get("end_time"),
                    dict.get("reverse"),
                    dict.get("limit_enable"),
                    dict.get("limit_value"),
                    dict.get("after_download"),
                    dict.get("gid_list"),
                ],
            )
            .unwrap();
    }

    // insert in to download_db_table in ghermez.db
    fn insertInDownloadTable(&self, list: Vec<HashMap<&str, &str>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let mut transaction = connection.transaction().unwrap();

        let transaction_size = 5;
        for (i, dict) in list.clone().into_iter().enumerate() {
            if i % transaction_size == 0 {
                transaction.commit().unwrap();
                transaction = connection.transaction().unwrap();
            }
            transaction
                .execute(
                    "
                INSERT INTO download_db_table VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
                )
                ",
                    [
                        dict.get("file_name"),
                        dict.get("status"),
                        dict.get("size"),
                        dict.get("downloaded_size"),
                        dict.get("percent"),
                        dict.get("connections"),
                        dict.get("rate"),
                        dict.get("estimate_time_left"),
                        dict.get("gid"),
                        dict.get("link"),
                        dict.get("first_try_date"),
                        dict.get("last_try_date"),
                        dict.get("category"),
                    ],
                )
                .unwrap();
        }
        transaction.commit().unwrap();

        // job is done! open the lock
        drop(connection);

        if !list.is_empty() {
            let dict = list.last().unwrap();

            // item must be inserted to gid_list of 'All Downloads' and gid_list of category
            // find download category and gid
            let category = dict.get("category").unwrap();

            // get category_dict from data base
            let mut category_dict = self.searchCategoryInCategoryTable(category).unwrap();

            // get all_downloads_dict from data base
            let mut all_downloads_dict =
                self.searchCategoryInCategoryTable("All Downloads").unwrap();

            // get gid_list
            let re = Regex::new(r"[\d\w]+").unwrap();
            let mut category_gid_list: Vec<_> = re
                .find_iter(category_dict.get("gid_list").unwrap())
                .map(|m| m.as_str())
                .collect();
            let mut all_downloads_gid_list: Vec<_> = re
                .find_iter(all_downloads_dict.get("gid_list").unwrap())
                .map(|m| m.as_str())
                .collect();

            for dict in list {
                let gid = dict.get("gid").unwrap();

                // add gid of item to gid_list
                category_gid_list.push(gid);
                all_downloads_gid_list.push(gid);
            }

            *category_dict.get_mut("gid_list").unwrap() = format!("{category_gid_list:?}");
            *all_downloads_dict.get_mut("gid_list").unwrap() =
                format!("{all_downloads_gid_list:?}");

            // update category_db_table
            self.updateCategoryTable(vec![all_downloads_dict]);
            self.updateCategoryTable(vec![category_dict]);
        }
    }

    // insert in addlink table in ghermez.db
    fn insertInAddLinkTable(&self, list: Vec<HashMap<&str, &str>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let mut transaction = connection.transaction().unwrap();

        let transaction_size = 5;
        for (i, dict) in list.clone().into_iter().enumerate() {
            if i % transaction_size == 0 {
                transaction.commit().unwrap();
                transaction = connection.transaction().unwrap();
            }

            // first column and after download column is NULL
            transaction
                .execute(
                    "
                    INSERT INTO addlink_db_table VALUES(NULL,
                        ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                        ?8, ?9, ?10, ?11, ?12, ?13,
                        ?14, ?15, ?16, ?17, ?18,
                        NULL
                    )
                ",
                    [
                        dict.get("gid"),
                        dict.get("out"),
                        dict.get("start_time"),
                        dict.get("end_time"),
                        dict.get("link"),
                        dict.get("ip"),
                        dict.get("port"),
                        dict.get("proxy_user"),
                        dict.get("proxy_passwd"),
                        dict.get("download_user"),
                        dict.get("download_passwd"),
                        dict.get("connections"),
                        dict.get("limit_value"),
                        dict.get("download_path"),
                        dict.get("referer"),
                        dict.get("load_cookies"),
                        dict.get("user_agent"),
                        dict.get("header"),
                    ],
                )
                .unwrap();
        }
        transaction.commit().unwrap();
    }

    fn insertInVideoFinderTable(&self, list: Vec<HashMap<&str, &str>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let mut transaction = connection.transaction().unwrap();

        let transaction_size = 5;
        for (i, dict) in list.clone().into_iter().enumerate() {
            if i % transaction_size == 0 {
                transaction.commit().unwrap();
                transaction = connection.transaction().unwrap();
            }

            // first column is NULL
            transaction
                .execute(
                    "
                        INSERT INTO video_finder_db_table VALUES(
                            NULL, ?1, ?2, ?3, ?4, ?5, ?6, ?7
                        )
                    ",
                    [
                        dict.get("video_gid"),
                        dict.get("audio_gid"),
                        dict.get("video_completed"),
                        dict.get("audio_completed"),
                        dict.get("muxing_status"),
                        dict.get("checking"),
                        dict.get("download_path"),
                    ],
                )
                .unwrap();
        }
        transaction.commit().unwrap();
    }

    fn searchGidInVideoFinderTable(&self, gid: &str) -> Option<HashMap<String, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let mut stmt = connection
            .prepare(
                "
                SELECT * FROM video_finder_db_table WHERE audio_gid = ?1 OR video_gid = ?2
                ",
            )
            .unwrap();

        let mut rows = stmt.query([gid, gid]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            return Some(HashMap::from([
                ("video_gid".to_string(), row.get(1).unwrap()),
                ("audio_gid".to_string(), row.get(2).unwrap()),
                ("video_completed".to_string(), row.get(3).unwrap()),
                ("audio_completed".to_string(), row.get(4).unwrap()),
                ("muxing_status".to_string(), row.get(5).unwrap()),
                ("checking".to_string(), row.get(6).unwrap()),
                ("download_path".to_string(), row.get(7).unwrap()),
            ]));
        }
        None
    }

    fn searchGidInDownloadTable(&self, gid: &str) -> Option<HashMap<String, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let mut stmt = connection
            .prepare(
                "
                SELECT * FROM download_db_table WHERE gid = ?1
                ",
            )
            .unwrap();

        let mut rows = stmt.query([gid]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            return Some(HashMap::from([
                ("file_name".to_string(), row.get(0).unwrap()),
                ("status".to_string(), row.get(1).unwrap()),
                ("size".to_string(), row.get(2).unwrap()),
                ("downloaded_size".to_string(), row.get(3).unwrap()),
                ("percent".to_string(), row.get(4).unwrap()),
                ("connections".to_string(), row.get(5).unwrap()),
                ("rate".to_string(), row.get(6).unwrap()),
                ("estimate_time_left".to_string(), row.get(7).unwrap()),
                ("gid".to_string(), row.get(8).unwrap()),
                ("link".to_string(), row.get(9).unwrap()),
                ("first_try_date".to_string(), row.get(10).unwrap()),
                ("last_try_date".to_string(), row.get(11).unwrap()),
                ("category".to_string(), row.get(12).unwrap()),
            ]));
        }
        None
    }

    // return all items in download_db_table
    // '*' for category, cause that method returns all items.
    fn returnItemsInDownloadTable(
        &self,
        category: Option<&str>,
    ) -> HashMap<String, HashMap<&str, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let query = if category.is_some() {
            format!(
                "SELECT * FROM download_db_table WHERE category = '{}'",
                category.unwrap()
            )
        } else {
            "SELECT * FROM download_db_table".to_string()
        };

        let mut stmt = connection.prepare(&query).unwrap();
        let rows = stmt
            .query_map([], |row| {
                // change format of tuple to dictionary
                Ok(HashMap::from([
                    ("file_name", row.get::<usize, String>(0).unwrap()),
                    ("status", row.get(1).unwrap()),
                    ("size", row.get(2).unwrap()),
                    ("downloaded_size", row.get(3).unwrap()),
                    ("percent", row.get(4).unwrap()),
                    ("connections", row.get(5).unwrap()),
                    ("rate", row.get(6).unwrap()),
                    ("estimate_time_left", row.get(7).unwrap()),
                    ("gid", row.get(8).unwrap()),
                    ("link", row.get(9).unwrap()),
                    ("first_try_date", row.get(10).unwrap()),
                    ("last_try_date", row.get(11).unwrap()),
                    ("category", row.get(12).unwrap()),
                ]))
            })
            .unwrap();

        let mut downloads_dict = HashMap::new();
        for download in rows {
            // add dict to the downloads_dict
            // gid is key and dict is value
            let download = download.unwrap();
            downloads_dict.insert(download.get("gid").unwrap().to_string(), download);
        }
        downloads_dict
    }

    // this method checks existence of a link in addlink_db_table
    fn searchLinkInAddLinkTable(&self, link: &str) -> bool {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let result = connection.execute("SELECT * FROM addlink_db_table WHERE link = (?1)", [link]);

        if result.is_ok_and(|r| r > 0) {
            return true;
        }
        false
    }

    fn searchGidInAddLinkTable(&self, gid: &str) -> Option<HashMap<String, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let mut stmt = connection
            .prepare(
                "
                SELECT * FROM addlink_db_table WHERE gid = ?1
                ",
            )
            .unwrap();

        let mut rows = stmt.query([gid]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            return Some(HashMap::from([
                ("gid".to_string(), row.get(1).unwrap_or("NULL".to_string())),
                ("out".to_string(), row.get(2).unwrap_or("NULL".to_string())),
                (
                    "start_time".to_string(),
                    row.get(3).unwrap_or("NULL".to_string()),
                ),
                (
                    "end_time".to_string(),
                    row.get(4).unwrap_or("NULL".to_string()),
                ),
                ("link".to_string(), row.get(5).unwrap_or("NULL".to_string())),
                ("ip".to_string(), row.get(6).unwrap_or("NULL".to_string())),
                ("port".to_string(), row.get(7).unwrap_or("NULL".to_string())),
                (
                    "proxy_user".to_string(),
                    row.get(8).unwrap_or("NULL".to_string()),
                ),
                (
                    "proxy_passwd".to_string(),
                    row.get(9).unwrap_or("NULL".to_string()),
                ),
                (
                    "download_user".to_string(),
                    row.get(10).unwrap_or("NULL".to_string()),
                ),
                (
                    "download_passwd".to_string(),
                    row.get(11).unwrap_or("NULL".to_string()),
                ),
                (
                    "connections".to_string(),
                    row.get(12).unwrap_or("NULL".to_string()),
                ),
                (
                    "limit_value".to_string(),
                    row.get(13).unwrap_or("NULL".to_string()),
                ),
                (
                    "download_path".to_string(),
                    row.get(14).unwrap_or("NULL".to_string()),
                ),
                (
                    "referer".to_string(),
                    row.get(15).unwrap_or("NULL".to_string()),
                ),
                (
                    "load_cookies".to_string(),
                    row.get(16).unwrap_or("NULL".to_string()),
                ),
                (
                    "user_agent".to_string(),
                    row.get(17).unwrap_or("NULL".to_string()),
                ),
                (
                    "header".to_string(),
                    row.get(18).unwrap_or("NULL".to_string()),
                ),
                (
                    "after_download".to_string(),
                    row.get(19).unwrap_or("NULL".to_string()),
                ),
            ]));
        }
        None
    }

    // return items in addlink_db_table
    // '*' for category, cause that method returns all items.
    fn returnItemsInAddLinkTable(
        &self,
        category: Option<&str>,
    ) -> HashMap<String, HashMap<String, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let query = if category.is_some() {
            format!(
                "SELECT * FROM addlink_db_table WHERE category = '{}'",
                category.unwrap()
            )
        } else {
            "SELECT * FROM addlink_db_table".to_string()
        };

        let mut stmt = connection.prepare(&query).unwrap();
        let rows = stmt
            .query_map([], |row| {
                // change format of tuple to dictionary
                Ok(HashMap::from([
                    ("gid".to_string(), row.get::<usize, String>(1).unwrap()),
                    ("out".to_string(), row.get(2).unwrap()),
                    ("start_time".to_string(), row.get(3).unwrap()),
                    ("end_time".to_string(), row.get(4).unwrap()),
                    ("link".to_string(), row.get(5).unwrap()),
                    ("ip".to_string(), row.get(6).unwrap()),
                    ("port".to_string(), row.get(7).unwrap()),
                    ("proxy_user".to_string(), row.get(8).unwrap()),
                    ("proxy_passwd".to_string(), row.get(9).unwrap()),
                    ("download_user".to_string(), row.get(10).unwrap()),
                    ("download_passwd".to_string(), row.get(11).unwrap()),
                    ("connections".to_string(), row.get(12).unwrap()),
                    ("limit_value".to_string(), row.get(13).unwrap()),
                    ("download_path".to_string(), row.get(14).unwrap()),
                    ("referer".to_string(), row.get(15).unwrap()),
                    ("load_cookies".to_string(), row.get(16).unwrap()),
                    ("user_agent".to_string(), row.get(17).unwrap()),
                    ("header".to_string(), row.get(18).unwrap()),
                    ("after_download".to_string(), row.get(19).unwrap()),
                ]))
            })
            .unwrap();

        let mut addlink_dict = HashMap::new();
        for download in rows {
            // add dict to the addlink_dict
            // gid as key and dict as value
            let download = download.unwrap();
            addlink_dict.insert(download.get("gid").unwrap().to_string(), download);
        }
        addlink_dict
    }

    // this method updates download_db_table
    fn updateDownloadTable(&self, list: Vec<HashMap<&str, &str>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        for dict in list {
            // update data base if value for the keys is not None
            transaction
                .execute(
                    "
                UPDATE download_db_table SET
                file_name = coalesce(?1, file_name),
                status = coalesce(?2, status),
                size = coalesce(?3, size),
                downloaded_size = coalesce(?4, downloaded_size),
                percent = coalesce(?5, percent),
                connections = coalesce(?6, connections),
                rate = coalesce(?7, rate),
                estimate_time_left = coalesce(?8, estimate_time_left),
                link = coalesce(?9, link),
                first_try_date = coalesce(?10, first_try_date),
                last_try_date = coalesce(?11, last_try_date),
                category = coalesce(?12, category)
                WHERE gid = ?13
            ",
                    [
                        dict.get("file_name"),
                        dict.get("status"),
                        dict.get("size"),
                        dict.get("downloaded_size"),
                        dict.get("percent"),
                        dict.get("connections"),
                        dict.get("rate"),
                        dict.get("estimate_time_left"),
                        dict.get("link"),
                        dict.get("first_try_date"),
                        dict.get("last_try_date"),
                        dict.get("category"),
                        dict.get("gid"),
                    ],
                )
                .unwrap();
        }
        transaction.commit().unwrap();
    }

    // this method updates category_db_table
    fn updateCategoryTable(&self, list: Vec<HashMap<&str, String>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        for dict in list {
            // update data base if value for the keys is not None
            transaction
                .execute(
                    "
                    UPDATE category_db_table SET
                    start_time_enable = coalesce(?1, start_time_enable),
                    start_time = coalesce(?2, start_time),
                    end_time_enable = coalesce(?3, end_time_enable),
                    end_time = coalesce(?4, end_time),
                    reverse = coalesce(?5, reverse),
                    limit_enable = coalesce(?6, limit_enable),
                    limit_value = coalesce(?7, limit_value),
                    after_download = coalesce(?8, after_download),
                    gid_list = coalesce(?9, gid_list)
                    WHERE category = ?10
                    ",
                    [
                        dict.get("start_time_enable"),
                        dict.get("start_time"),
                        dict.get("end_time_enable"),
                        dict.get("end_time"),
                        dict.get("reverse"),
                        dict.get("limit_enable"),
                        dict.get("limit_value"),
                        dict.get("after_download"),
                        dict.get("gid_list"),
                        dict.get("category"),
                    ],
                )
                .unwrap();
        }
        transaction.commit().unwrap();
    }

    fn updateAddLinkTable(&self, list: Vec<HashMap<&str, &str>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        for dict in list {
            // update data base if value for the keys is not None
            transaction
                .execute(
                    "
                    UPDATE addlink_db_table SET
                    out = coalesce(?1, out),
                    start_time = coalesce(?2, start_time),
                    end_time = coalesce(?3, end_time),
                    link = coalesce(?4, link),
                    ip = coalesce(?5, ip),
                    port = coalesce(?6, port),
                    proxy_user = coalesce(?7, proxy_user),
                    proxy_passwd = coalesce(?8, proxy_passwd),
                    download_user = coalesce(?9, download_user),
                    download_passwd = coalesce(?10, download_passwd),
                    connections = coalesce(?11, connections),
                    limit_value = coalesce(?12, limit_value),
                    download_path = coalesce(?13, download_path),
                    referer = coalesce(?14, referer),
                    load_cookies = coalesce(?15, load_cookies),
                    user_agent = coalesce(?16, user_agent),
                    header = coalesce(?17, header),
                    after_download = coalesce(?18 , after_download)
                    WHERE gid = ?19
                    ",
                    [
                        dict.get("out"),
                        dict.get("start_time"),
                        dict.get("end_time"),
                        dict.get("link"),
                        dict.get("ip"),
                        dict.get("port"),
                        dict.get("proxy_user"),
                        dict.get("proxy_passwd"),
                        dict.get("download_user"),
                        dict.get("download_passwd"),
                        dict.get("connections"),
                        dict.get("limit_value"),
                        dict.get("download_path"),
                        dict.get("referer"),
                        dict.get("load_cookies"),
                        dict.get("user_agent"),
                        dict.get("header"),
                        dict.get("after_download"),
                        dict.get("gid"),
                    ],
                )
                .unwrap();
        }
        transaction.commit().unwrap();
    }

    fn updateVideoFinderTable(&self, list: Vec<HashMap<&str, &str>>) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        for dict in list {
            if dict.contains_key("video_gid") {
                // update data base if value for the keys is not None
                transaction
                    .execute(
                        "
                        UPDATE video_finder_db_table SET
                        video_completed = coalesce(?1, video_completed),
                        audio_completed = coalesce(?2, audio_completed),
                        muxing_status = coalesce(?3, muxing_status),
                        checking = coalesce(?4, checking),
                        download_path = coalesce(?5, download_path)
                        WHERE video_gid = ?6
                        ",
                        [
                            dict.get("video_completed"),
                            dict.get("audio_completed"),
                            dict.get("muxing_status"),
                            dict.get("checking"),
                            dict.get("download_path"),
                            dict.get("video_gid"),
                        ],
                    )
                    .unwrap();
            } else if dict.contains_key("audio_gid") {
                // update data base if value for the keys is not None
                transaction
                    .execute(
                        "
                        UPDATE video_finder_db_table SET
                        video_completed = coalesce(?1, video_completed),
                        audio_completed = coalesce(?2, audio_completed),
                        muxing_status = coalesce(?3, muxing_status),
                        checking = coalesce(?4, checking),
                        download_path = coalesce(?5, download_path)
                        WHERE audio_gid = ?6
                        ",
                        [
                            dict.get("video_completed"),
                            dict.get("audio_completed"),
                            dict.get("muxing_status"),
                            dict.get("checking"),
                            dict.get("download_path"),
                            dict.get("audio_gid"),
                        ],
                    )
                    .unwrap();
            }
        }
        transaction.commit().unwrap();
    }

    fn setDefaultGidInAddlinkTable(
        &self,
        gid: &str,
        start_time: bool,
        end_time: bool,
        after_download: bool,
    ) {
        // lock data base
        let connection = self.connection.lock().unwrap();

        if start_time {
            connection
                .execute(
                    "
                    UPDATE addlink_db_table SET
                    start_time = NULL
                    WHERE gid = ?1
                ",
                    [gid],
                )
                .unwrap();
        }
        if end_time {
            connection
                .execute(
                    "
                    UPDATE addlink_db_table SET
                    end_time = NULL
                    WHERE gid = ?1
                ",
                    [gid],
                )
                .unwrap();
        }
        if after_download {
            connection
                .execute(
                    "
                    UPDATE addlink_db_table SET
                    after_download = NULL
                    WHERE gid = ?1
                ",
                    [gid],
                )
                .unwrap();
        }
    }

    fn searchCategoryInCategoryTable(&self, category: &str) -> Option<HashMap<&str, String>> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let mut stmt = connection
            .prepare(
                "
                SELECT * FROM category_db_table WHERE category = ?1
                ",
            )
            .unwrap();

        let mut rows = stmt.query([category]).unwrap();
        if let Some(row) = rows.next().unwrap() {
            return Some(HashMap::from([
                ("category", row.get(0).unwrap()),
                ("start_time_enable", row.get(1).unwrap()),
                ("start_time", row.get(2).unwrap()),
                ("end_time_enable", row.get(3).unwrap()),
                ("end_time", row.get(4).unwrap()),
                ("reverse", row.get(5).unwrap()),
                ("limit_enable", row.get(6).unwrap()),
                ("limit_value", row.get(7).unwrap()),
                ("after_download", row.get(8).unwrap()),
                ("gid_list", row.get(9).unwrap()),
            ]));
        }
        None
    }

    // return categories name
    fn categoriesList(&self) -> Vec<String> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let mut stmt = connection
            .prepare("SELECT category FROM category_db_table ORDER BY ROWID")
            .unwrap();

        let mut queues_list = vec![];

        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            queues_list.push(row.get(0).unwrap());
        }
        queues_list
    }

    fn setDBTablesToDefaultValue(&self) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        // change start_time_enable , end_time_enable , reverse ,
        // limit_enable , after_download value to default value !
        transaction
            .execute(
                "
                UPDATE category_db_table SET start_time_enable = 'no', end_time_enable = 'no',
                reverse = 'no', limit_enable = 'no', after_download = 'no'
            ",
                (),
            )
            .unwrap();

        // change status of download to 'stopped' if status isn't 'complete' or 'error'
        transaction
            .execute("
                UPDATE download_db_table SET status = 'stopped' WHERE status NOT IN ('complete', 'error')
            ", ())
            .unwrap();

        // change start_time and end_time and
        // after_download value to None in addlink_db_table!
        transaction
            .execute(
                "
                UPDATE addlink_db_table SET start_time = NULL,
                end_time = NULL, after_download = NULL
            ",
                (),
            )
            .unwrap();

        // change checking value to no in video_finder_db_table
        transaction
            .execute(
                "
                UPDATE video_finder_db_table SET checking = 'no'
            ",
                (),
            )
            .unwrap();

        transaction.commit().unwrap();
    }

    fn findActiveDownloads(&self, category: Option<&str>) -> Vec<String> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        // find download items is download_db_table with status = "downloading" or "waiting" or paused or scheduled
        let query = if category.is_some() {
            format!(
                "
            SELECT gid FROM download_db_table WHERE (category = '{}')
            AND (status = 'downloading' OR status = 'waiting'
            OR status = 'scheduled' OR status = 'paused')
            ",
                category.unwrap()
            )
        } else {
            "SELECT gid FROM download_db_table WHERE
            (status = 'downloading' OR status = 'waiting'
            OR status = 'scheduled' OR status = 'paused')"
                .to_string()
        };
        let mut stmt = connection.prepare(&query).unwrap();

        let mut gid_list = vec![];

        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            gid_list.push(row.get(0).unwrap());
        }

        gid_list
    }

    // this method returns items with 'downloading' or 'waiting' status
    fn returnDownloadingItems(&self) -> Vec<String> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        // find download items is download_db_table with status = "downloading" or "waiting" or paused or scheduled
        let mut stmt = connection
            .prepare(
                "
                SELECT gid FROM download_db_table WHERE
                (status = 'downloading' OR status = 'waiting')
            ",
            )
            .unwrap();

        let mut gid_list = vec![];

        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            gid_list.push(row.get(0).unwrap());
        }

        gid_list
    }

    // this method returns items with 'paused' status.
    fn returnPausedItems(&self) -> Vec<String> {
        // lock data base
        let connection = self.connection.lock().unwrap();

        // find download items is download_db_table with status = "downloading" or "waiting" or paused or scheduled
        let mut stmt = connection
            .prepare(
                "
                SELECT gid FROM download_db_table WHERE (status = 'paused')
            ",
            )
            .unwrap();

        let mut gid_list = vec![];

        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            gid_list.push(row.get(0).unwrap());
        }

        gid_list
    }

    // return all video_gids and audio_gids in video_finder_db_table
    fn returnVideoFinderGids(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        // lock data base
        let connection = self.connection.lock().unwrap();

        let mut stmt = connection
            .prepare(
                "
                SELECT video_gid, audio_gid FROM video_finder_db_table
            ",
            )
            .unwrap();

        let mut gid_list: Vec<String> = vec![];
        let mut video_gid_list: Vec<String> = vec![];
        let mut audio_gid_list: Vec<String> = vec![];

        let mut rows = stmt.query([]).unwrap();
        while let Some(row) = rows.next().unwrap() {
            gid_list.push(row.get(0).unwrap());
            video_gid_list.push(row.get(0).unwrap());

            gid_list.push(row.get(1).unwrap());
            audio_gid_list.push(row.get(1).unwrap());
        }
        (gid_list, video_gid_list, audio_gid_list)
    }

    // This method deletes a category from category_db_table
    fn deleteCategory(&self, category: &str) {
        // delete gids of this category from gid_list of 'All Downloads'
        let category_dict = self.searchCategoryInCategoryTable(category).unwrap();
        let mut all_downloads_dict = self.searchCategoryInCategoryTable("All Downloads").unwrap();

        // get gid_list
        let re = Regex::new(r"[\d\w]+").unwrap();
        let category_gid_list: Vec<_> = re
            .find_iter(category_dict.get("gid_list").unwrap())
            .map(|m| m.as_str())
            .collect();
        let all_downloads_gid_list: Vec<_> = re
            .find_iter(all_downloads_dict.get("gid_list").unwrap())
            .map(|m| m.as_str())
            .collect();
        let mut new_all_downloads_gid_list = all_downloads_gid_list.clone();

        // delete item from all_downloads_gid_list
        for gid in category_gid_list {
            new_all_downloads_gid_list.remove(
                new_all_downloads_gid_list
                    .iter()
                    .position(|x| *x == gid)
                    .unwrap(),
            );
        }

        // update category_db_table
        *all_downloads_dict.get_mut("gid_list").unwrap() =
            format!("{new_all_downloads_gid_list:?}");
        self.updateCategoryTable(vec![all_downloads_dict]);

        // lock data base
        let connection = self.connection.lock().unwrap();

        // delete category from data_base
        connection
            .execute(
                "
                DELETE FROM category_db_table WHERE category = ?1
            ",
                [category],
            )
            .unwrap();
    }

    // this method deletes all items in data_base
    fn resetDataBase(&self) {
        // update gid_list in categories with empty gid_list
        let all_downloads_dict = HashMap::from([
            ("category", "All Downloads".to_string()),
            ("gid_list", "[]".to_string()),
        ]);
        let single_downloads_dict = HashMap::from([
            ("category", "Single Downloads".to_string()),
            ("gid_list", "[]".to_string()),
        ]);
        let scheduled_downloads_dict = HashMap::from([
            ("category", "Scheduled Downloads".to_string()),
            ("gid_list", "[]".to_string()),
        ]);

        self.updateCategoryTable(vec![
            all_downloads_dict,
            single_downloads_dict,
            scheduled_downloads_dict,
        ]);

        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        // delete all items in category_db_table, except 'All Downloads' and 'Single Downloads'
        transaction.execute("
        DELETE FROM category_db_table WHERE category NOT IN ('All Downloads', 'Single Downloads', 'Scheduled Downloads')
        ", ())
        .unwrap();
        transaction
            .execute("DELETE FROM download_db_table", ())
            .unwrap();
        transaction
            .execute("DELETE FROM addlink_db_table", ())
            .unwrap();
        transaction.commit().unwrap();
    }

    // This method deletes a download item from download_db_table
    fn deleteItemInDownloadTable(&self, gid: &str, category: &str) {
        // lock data base
        let connection = self.connection.lock().unwrap();

        connection
            .execute(
                "
                DELETE FROM download_db_table WHERE gid = ?1
            ",
                [gid],
            )
            .unwrap();

        // job is done! open the lock
        drop(connection);

        // delete item from gid_list in category and All Downloads
        for category_name in [category, "All Downloads"] {
            let category_dict = self.searchCategoryInCategoryTable(category_name).unwrap();

            // get gid_list
            let re = Regex::new(r"\d+").unwrap();
            let gid_list: Vec<_> = re
                .find_iter(category_dict.get("gid_list").unwrap())
                .map(|m| m.as_str())
                .collect();
            let mut new_gid_list = gid_list.clone();

            // delete item
            for gid in &gid_list {
                new_gid_list.remove(new_gid_list.iter().position(|x| x == gid).unwrap());

                // if gid is in video_finder_db_table, both of video_gid and audio_gid must be deleted from gid_list
                let video_finder_dictionary = self.searchGidInVideoFinderTable(gid);

                if let Some(video_finder_dictionary) = video_finder_dictionary {
                    let video_gid = video_finder_dictionary.get("video_gid").unwrap();
                    let audio_gid = video_finder_dictionary.get("audio_gid").unwrap();

                    if gid == video_gid {
                        new_gid_list
                            .remove(new_gid_list.iter().position(|x| x == audio_gid).unwrap());
                    } else {
                        new_gid_list
                            .remove(new_gid_list.iter().position(|x| x == video_gid).unwrap());
                    }
                }

                // update category_db_table
                let mut new_category_dict = category_dict.clone();
                *new_category_dict.get_mut("gid_list").unwrap() = format!("{new_gid_list:?}");
                self.updateCategoryTable(vec![new_category_dict]);
            }
        }
    }

    // this method replaces:
    // GB >> GiB
    // MB >> MiB
    // KB >> KiB
    // Read this link for more information:
    // https://en.wikipedia.org/wiki/Orders_of_magnitude_(data)
    fn correctDataBase(&self) {
        // lock data base
        let mut connection = self.connection.lock().unwrap();
        let transaction = connection.transaction().unwrap();

        for units in [["KB", "KiB"], ["MB", "MiB"], ["GB", "GiB"]] {
            let dict = HashMap::from([("old_unit", units[0]), ("new_unit", units[1])]);

            transaction
                .execute(
                    "
                    UPDATE download_db_table 
                    SET size = replace(size, ?1, ?2)
                ",
                    [dict.get("old_unit").unwrap(), dict.get("new_unit").unwrap()],
                )
                .unwrap();
            transaction
                .execute(
                    "
                    UPDATE download_db_table
                    SET rate = replace(rate, ?1, ?2)
                ",
                    [dict.get("old_unit").unwrap(), dict.get("new_unit").unwrap()],
                )
                .unwrap();
            transaction
                .execute(
                    "
                UPDATE download_db_table 
                SET downloaded_size = replace(downloaded_size, ?1, ?2)
                ",
                    [dict.get("old_unit").unwrap(), dict.get("new_unit").unwrap()],
                )
                .unwrap();
        }
        transaction.commit().unwrap();
    }
}
