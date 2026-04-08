async def fetch_data():  # unasync: generate @mytag
    result = await get_data()
    return result

def fetch_data():  # unasync: generated @mytag
    result = get_data()
    return result
