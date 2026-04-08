async def fetch_data():  # unasync: generate @sync
    result = await get_data()
    return result

def fetch_data():  # unasync: generated @sync
    result = get_data()
    return result
