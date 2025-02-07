TABLE: users
(pk) user_id: uuid
user_email: string 
user_password: string
...

TABLE: chats
(pk) chat_id: uuid 
(fk) user_id: uuid

TABLE: chat_actions
(pk) chat_action_id: uuid
(fk) chat_id: uuid

TABLE: chat_action_step
(pk) chat_action_step_id: uuid
(fk) chat_action_id: uuid
msg_type: ENUM.MSGTYPES
content: string