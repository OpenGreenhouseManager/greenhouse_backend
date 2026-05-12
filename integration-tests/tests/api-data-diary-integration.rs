use greenhouse_core::data_storage_service_dto::diary_dtos::get_diary::GetDiaryResponseDto;
use greenhouse_core::data_storage_service_dto::diary_dtos::get_diary_entry::DiaryEntryResponseDto;
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_entry::PostDiaryEntryDtoRequest;
use greenhouse_core::data_storage_service_dto::diary_dtos::post_diary_tag::PostDiaryTagDtoRequest;
use greenhouse_core::data_storage_service_dto::diary_dtos::put_diary_entry::PutDiaryEntryDtoRequest;
use test_helper::TestContext;
mod test_helper;

#[tokio::test]
async fn test_create_and_get_diary_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Create diary entry
    let client = reqwest::Client::new();
    let post_entry = PostDiaryEntryDtoRequest {
        title: "Test Entry".to_string(),
        content: "This is a test diary entry.".to_string(),
        date: chrono::Utc::now().to_string(),
    };
    let response = client
        .post("http://localhost:3000/api/diary")
        .json(&post_entry)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to create diary entry"
    );

    // Get diary entries for a date range
    let response = client
        .get(format!(
            "http://localhost:3000/api/diary/{}/{}",
            post_entry.date, post_entry.date
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to get diary entries"
    );
    let entry = response.json::<GetDiaryResponseDto>().await.unwrap();
    assert!(!entry.entries.is_empty(), "No diary entries returned");

    // Get a specific diary entry by id
    let response = client
        .get(format!(
            "http://localhost:3000/api/diary/{}",
            entry.entries[0].id
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to get diary entry by id"
    );

    context.stop().await;
}

#[tokio::test]
async fn test_update_diary_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    // Create diary entry
    let client = reqwest::Client::new();
    let post_entry = PostDiaryEntryDtoRequest {
        title: "Entry to Update".to_string(),
        content: "Original content.".to_string(),
        date: chrono::Utc::now().to_string(),
    };
    let response = client
        .post("http://localhost:3000/api/diary")
        .json(&post_entry)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to create diary entry"
    );

    // Get diary entries for a date range
    let response = client
        .get(format!(
            "http://localhost:3000/api/diary/{}/{}",
            post_entry.date, post_entry.date
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    let entry = response.json::<GetDiaryResponseDto>().await.unwrap();

    // Update the diary entry
    let update_entry = PutDiaryEntryDtoRequest {
        title: String::from("Updated Title"),
        content: String::from("Updated content."),
        date: chrono::Utc::now().to_string(),
    };
    let response = client
        .put(format!(
            "http://localhost:3000/api/diary/{}",
            entry.entries[0].id
        ))
        .json(&update_entry)
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to update diary entry"
    );

    // Get the updated diary entry
    let response = client
        .get(format!(
            "http://localhost:3000/api/diary/{}",
            entry.entries[0].id
        ))
        .header("Access-Control-Allow-Credentials", "true")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    let entry: serde_json::Value = response.json().await.unwrap();
    assert_eq!(entry["title"], "Updated Title");
    assert_eq!(entry["content"], "Updated content.");

    context.stop().await;
}

#[tokio::test]
async fn test_add_and_get_tags_on_diary_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();
    let post_entry = PostDiaryEntryDtoRequest {
        title: "Tagged Entry".to_string(),
        content: "Entry with tags.".to_string(),
        date: chrono::Utc::now().to_string(),
    };
    let response = client
        .post("http://localhost:3000/api/diary")
        .json(&post_entry)
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "Failed to create diary entry"
    );
    let entry: DiaryEntryResponseDto = response.json().await.unwrap();
    assert!(entry.tags.is_empty(), "New entry should have no tags");

    // Add a tag
    let response = client
        .post(format!("http://localhost:3000/api/diary/{}/tags", entry.id))
        .json(&PostDiaryTagDtoRequest {
            tag_name: "rust".to_string(),
        })
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to add tag");

    // Get entry and verify tag is present
    let response = client
        .get(format!("http://localhost:3000/api/diary/{}", entry.id))
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to get diary entry");
    let entry: DiaryEntryResponseDto = response.json().await.unwrap();
    assert_eq!(entry.tags, vec!["rust"], "Tag should be present on entry");

    context.stop().await;
}

#[tokio::test]
async fn test_remove_tag_from_diary_entry() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();
    let post_entry = PostDiaryEntryDtoRequest {
        title: "Entry for Tag Removal".to_string(),
        content: "Will have tag removed.".to_string(),
        date: chrono::Utc::now().to_string(),
    };
    let response = client
        .post("http://localhost:3000/api/diary")
        .json(&post_entry)
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    let entry: DiaryEntryResponseDto = response.json().await.unwrap();

    // Add tag
    client
        .post(format!("http://localhost:3000/api/diary/{}/tags", entry.id))
        .json(&PostDiaryTagDtoRequest {
            tag_name: "temporary".to_string(),
        })
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();

    // Remove the tag
    let response = client
        .delete(format!(
            "http://localhost:3000/api/diary/{}/tags/temporary",
            entry.id
        ))
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Failed to remove tag");

    // Verify tag is gone
    let response = client
        .get(format!("http://localhost:3000/api/diary/{}", entry.id))
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    let entry: DiaryEntryResponseDto = response.json().await.unwrap();
    assert!(entry.tags.is_empty(), "Tags should be empty after removal");

    context.stop().await;
}

#[tokio::test]
async fn test_search_diary_entries_by_partial_tag() {
    let mut context = TestContext::new();
    context.start_all_services().await;
    let token = test_helper::admin_login().await;

    let client = reqwest::Client::new();

    // Create two entries
    let create = |title: &str| PostDiaryEntryDtoRequest {
        title: title.to_string(),
        content: "content".to_string(),
        date: chrono::Utc::now().to_string(),
    };
    let r1 = client
        .post("http://localhost:3000/api/diary")
        .json(&create("Entry A"))
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    let entry_a: DiaryEntryResponseDto = r1.json().await.unwrap();

    let r2 = client
        .post("http://localhost:3000/api/diary")
        .json(&create("Entry B"))
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    let entry_b: DiaryEntryResponseDto = r2.json().await.unwrap();

    // Tag entry A with "greenhouse-rust", entry B with "unrelated"
    for (id, tag) in [(&entry_a.id, "greenhouse-rust"), (&entry_b.id, "unrelated")] {
        client
            .post(format!("http://localhost:3000/api/diary/{id}/tags"))
            .json(&PostDiaryTagDtoRequest {
                tag_name: tag.to_string(),
            })
            .header("Cookie", format!("auth-token={token}"))
            .send()
            .await
            .unwrap();
    }

    // Search by partial name "greenhouse" — should return only entry A
    let response = client
        .get("http://localhost:3000/api/diary/tags/greenhouse")
        .header("Cookie", format!("auth-token={token}"))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success(), "Tag search failed");
    let result: GetDiaryResponseDto = response.json().await.unwrap();
    assert_eq!(result.entries.len(), 1, "Should find exactly one entry");
    assert_eq!(result.entries[0].id, entry_a.id, "Should return Entry A");

    context.stop().await;
}
