async def fetch_data():  # unasync: generate @mytag
    result = await get_data()
    return result
