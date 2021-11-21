
server:
    post: /{{table_id}}/answer
        payload:
            {
                "player": str,
                "answer": str
            }
        respond:
            {
                "answer_ok" : bool
            }
            
client
    post: /{{table_id}}/turnend
        payload:
            {
                "player_winner": str
            }
    post: /{{table_id}}/turnstart
        payload:
            [
                "{{card_id_1}}",
                "{{card_id_2}}",
                "{{card_id_3}}",
                "{{card_id_4}}"
            ]