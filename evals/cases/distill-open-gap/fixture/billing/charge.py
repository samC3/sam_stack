class ChargeFailed(Exception):
    pass


def charge(card, amount_cents, gateway):
    result = gateway.charge(card.token, amount_cents)
    if not result.ok:
        raise ChargeFailed(result.reason)
    return result.id
