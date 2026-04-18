async def fetch_data():  # unasync: generate @sync
    result = await get_data()
    return result

async def fetch_data():  # unasync: generated @sync
    result = await get_data()
    return result
