def in_operator(x, y):
    if type(y) == type:
        if isinstance(x, y):
            return True
        # TODO: trait check
        return False
    elif (type(y) == list or type(y) == set) and type(y[0]) == type:
        # FIXME:
        type_check = in_operator(x[0], y[0])
        len_check = len(x) == len(y)
        return type_check and len_check
    elif type(y) == dict and type(y[0]) == type:
        NotImplemented
    else:
        return x in y