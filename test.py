def calculate_bmi_result(height_cm, weight_kg):
    bmi_approx = (weight_kg * 10000) / (height_cm * height_cm)

    if bmi_approx < 18.5 * 10:
        category_code = 1
    elif bmi_approx < 25.0 * 10:
        category_code = 2
    else:
        category_code = 3

    if bmi_approx / 10 > 25:
        bmi_int = 25
    else:
        bmi_int = int(bmi_approx / 10)

    result = (bmi_int * 10) + category_code

    if result > 255:
        result = 255

    return result


height_cm = 170
weight_kg = 65
result = calculate_bmi_result(height_cm, weight_kg)
print(result)