async def fetch_data():  # unasync: generate @sync
    result = await get_data()
    return result
