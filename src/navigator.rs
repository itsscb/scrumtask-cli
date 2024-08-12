use anyhow::{anyhow, Context, Ok, Result};
use std::rc::Rc;

use crate::{
    db::JiraDatabase,
    models::Action,
    ui::{EpicDetail, HomePage, Page, Prompts, StoryDetail},
};

pub struct Navigator {
    pages: Vec<Box<dyn Page>>,
    prompts: Prompts,
    db: Rc<JiraDatabase>,
}

impl Navigator {
    pub fn new(db: Rc<JiraDatabase>) -> Self {
        Self {
            pages: vec![Box::new(HomePage::new(Rc::clone(&db)))],
            prompts: Prompts::new(),
            db,
        }
    }

    pub fn get_current_page(&self) -> Option<&Box<dyn Page>> {
        self.pages.last()
    }

    pub fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::NavigateToEpicDetail { epic_id } => {
                // create a new EpicDetail instance and add it to the pages vector
                self.pages.push(Box::new(EpicDetail {
                    epic_id,
                    db: Rc::clone(&self.db),
                }));
            }
            Action::NavigateToStoryDetail { epic_id, story_id } => {
                // create a new StoryDetail instance and add it to the pages vector
                self.pages.push(Box::new(StoryDetail {
                    epic_id,
                    story_id,
                    db: Rc::clone(&self.db),
                }));
            }
            Action::NavigateToPreviousPage => {
                // remove the last page from the pages vector
                let _ = self.pages.pop();
            }
            Action::CreateEpic => {
                // prompt the user to create a new epic and persist it in the database
                self.db
                    .create_epic((self.prompts.create_epic)())
                    .with_context(|| format!("failed to create epic"))?;
            }
            Action::UpdateEpicStatus { epic_id } => {
                // prompt the user to update status and persist it in the database
                let status = (self.prompts.update_status)()
                    .with_context(|| format!("invalid status: {epic_id}"))?;
                self.db.update_epic_status(epic_id, status)?;
            }
            Action::DeleteEpic { epic_id } => {
                // prompt the user to delete the epic and persist it in the database
                self.db
                    .delete_epic(epic_id)
                    .with_context(|| format!("failed to delete epic: {epic_id}"))?;
            }
            Action::CreateStory { epic_id } => {
                // prompt the user to create a new story and persist it in the database
                self.db
                    .create_story((self.prompts.create_story)(), epic_id)
                    .with_context(|| format!("failed to create story: {epic_id}"))?;
            }
            Action::UpdateStoryStatus { story_id } => {
                // prompt the user to update status and persist it in the database
                let status = (self.prompts.update_status)()
                    .with_context(|| format!("invalid status: {story_id}"))?;
                self.db.update_story_status(story_id, status)?;
            }
            Action::DeleteStory { epic_id, story_id } => {
                // prompt the user to delete the story and persist it in the database
                if (self.prompts.delete_story)() {
                    self.db
                        .delete_story(epic_id, story_id)
                        .with_context(|| format!("failed to delete story: {story_id}"))?;
                }
            }
            Action::Exit => {
                // remove all pages from the pages vector
                self.pages.clear();
            }
        }

        Ok(())
    }

    // Private functions used for testing

    fn get_page_count(&self) -> usize {
        self.pages.len()
    }

    fn set_prompts(&mut self, prompts: Prompts) {
        self.prompts = prompts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::test_utils::MockDB,
        models::{Epic, Status, Story},
    };

    #[test]
    fn should_start_on_home_page() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let nav = Navigator::new(db);

        assert_eq!(nav.get_page_count(), 1);

        let current_page = nav.get_current_page().unwrap();
        let home_page = current_page.as_any().downcast_ref::<HomePage>();

        assert_eq!(home_page.is_some(), true);
    }

    #[test]
    fn handle_action_should_navigate_pages() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });

        let mut nav = Navigator::new(db);

        nav.handle_action(Action::NavigateToEpicDetail { epic_id: 1 })
            .unwrap();
        assert_eq!(nav.get_page_count(), 2);

        let current_page = nav.get_current_page().unwrap();
        let epic_detail_page = current_page.as_any().downcast_ref::<EpicDetail>();
        assert_eq!(epic_detail_page.is_some(), true);

        nav.handle_action(Action::NavigateToStoryDetail {
            epic_id: 1,
            story_id: 2,
        })
        .unwrap();
        assert_eq!(nav.get_page_count(), 3);

        let current_page = nav.get_current_page().unwrap();
        let story_detail_page = current_page.as_any().downcast_ref::<StoryDetail>();
        assert_eq!(story_detail_page.is_some(), true);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 2);

        let current_page = nav.get_current_page().unwrap();
        let epic_detail_page = current_page.as_any().downcast_ref::<EpicDetail>();
        assert_eq!(epic_detail_page.is_some(), true);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 1);

        let current_page = nav.get_current_page().unwrap();
        let home_page = current_page.as_any().downcast_ref::<HomePage>();
        assert_eq!(home_page.is_some(), true);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 0);

        nav.handle_action(Action::NavigateToPreviousPage).unwrap();
        assert_eq!(nav.get_page_count(), 0);
    }

    #[test]
    fn handle_action_should_clear_pages_on_exit() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });

        let mut nav = Navigator::new(db);

        nav.handle_action(Action::NavigateToEpicDetail { epic_id: 1 })
            .unwrap();
        nav.handle_action(Action::NavigateToStoryDetail {
            epic_id: 1,
            story_id: 2,
        })
        .unwrap();
        nav.handle_action(Action::Exit).unwrap();

        assert_eq!(nav.get_page_count(), 0);
    }

    #[test]
    fn handle_action_should_handle_create_epic() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.create_epic = Box::new(|| Epic::new("name".to_owned(), "description".to_owned()));

        nav.set_prompts(prompts);

        nav.handle_action(Action::CreateEpic).unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.epics.len(), 1);

        let epic = db_state.epics.into_iter().next().unwrap().1;
        assert_eq!(epic.name, "name".to_owned());
        assert_eq!(epic.description, "description".to_owned());
    }

    #[test]
    fn handle_action_should_handle_update_epic() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.update_status = Box::new(|| Some(Status::InProgress));

        nav.set_prompts(prompts);

        nav.handle_action(Action::UpdateEpicStatus { epic_id })
            .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(
            db_state.epics.get(&epic_id).unwrap().status,
            Status::InProgress
        );
    }

    #[test]
    fn handle_action_should_handle_delete_epic() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.delete_epic = Box::new(|| true);

        nav.set_prompts(prompts);

        nav.handle_action(Action::DeleteEpic { epic_id }).unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.epics.len(), 0);
    }

    #[test]
    fn handle_action_should_handle_create_story() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.create_story = Box::new(|| Story::new("name".to_owned(), "description".to_owned()));

        nav.set_prompts(prompts);

        nav.handle_action(Action::CreateStory { epic_id }).unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.stories.len(), 1);

        let story = db_state.stories.into_iter().next().unwrap().1;
        assert_eq!(story.name, "name".to_owned());
        assert_eq!(story.description, "description".to_owned());
    }

    #[test]
    fn handle_action_should_handle_update_story() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();
        let story_id = db
            .create_story(Story::new("".to_owned(), "".to_owned()), epic_id)
            .unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.update_status = Box::new(|| Some(Status::InProgress));

        nav.set_prompts(prompts);

        nav.handle_action(Action::UpdateStoryStatus { story_id })
            .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(
            db_state.stories.get(&story_id).unwrap().status,
            Status::InProgress
        );
    }

    #[test]
    fn handle_action_should_handle_delete_story() {
        let db = Rc::new(JiraDatabase {
            database: Box::new(MockDB::new()),
        });
        let epic_id = db
            .create_epic(Epic::new("".to_owned(), "".to_owned()))
            .unwrap();
        let story_id = db
            .create_story(Story::new("".to_owned(), "".to_owned()), epic_id)
            .unwrap();

        let mut nav = Navigator::new(Rc::clone(&db));

        let mut prompts = Prompts::new();
        prompts.delete_story = Box::new(|| true);

        nav.set_prompts(prompts);

        nav.handle_action(Action::DeleteStory { epic_id, story_id })
            .unwrap();

        let db_state = db.read_db().unwrap();
        assert_eq!(db_state.stories.len(), 0);
    }
}
