import aiohttp
import asyncio
import sqlite3
from urllib.parse import urlparse

async def get_character_info(character_id, max_retries=5):
    query = '''
    query ($id: Int) {
        Character(id: $id) {
            name {
                full
            }
            image {
                medium
            }
        }
    }
    '''
    variables = {"id": character_id}
    url = 'https://graphql.anilist.co'
    
    retry_count = 0

    # Use aiohttp to make an async request
    async with aiohttp.ClientSession() as session:
        while retry_count < max_retries:
            async with session.post(url, json={'query': query, 'variables': variables}) as response:
                if response.status == 200:
                    data = await response.json()
                    character = data['data']['Character']
                    return character['name']['full'], character['image']['medium']
                elif response.status == 429:
                    print("Rate limit hit. Waiting 30 seconds before retrying...")
                    await asyncio.sleep(30)
                    retry_count += 1
                else:
                    print(f"Error {response.status}: {await response.text()}")
                    break

    print("Failed to fetch after retries.")
    return None, None

async def add_character(character_id, name, image_url):
    try:
        con = sqlite3.connect("databases/swapAnniversary.db")
        cur = con.cursor()

        # Check for existing character ID
        cur.execute("SELECT 1 FROM characters WHERE character_id = ?", (character_id,))
        if cur.fetchone():
            print(f"Character ID {character_id} already exists. Skipping.")
            con.close()
            return

        # Insert if not found
        cur.execute("""
            INSERT INTO characters (character_id, character_name, character_image)
            VALUES (?, ?, ?);
        """, (character_id, name, image_url))
        
        con.commit()
        con.close()
        print(f"Character {name} added successfully.")
    except sqlite3.IntegrityError as e:
        print(f"Database integrity error: {e}")
    except Exception as e:
        print(f"Error adding character: {e}")


async def main():
    with open("characters.txt") as file:
        for line in file:
            url = line.rstrip()
            _, _, path, _, _, _ = urlparse(url)
            split = path.split("/")
            if len(split) < 3:  # Avoid invalid URLs
                continue
            character_id = int(split[2])
            name, image = await get_character_info(character_id)
            if name and image:
                await add_character(character_id, name, image)

if __name__ == "__main__":
    asyncio.run(main())
