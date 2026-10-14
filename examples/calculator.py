# @decision Menggunakan pendekatan komputasi presisi tinggi untuk kalkulator
# @constraint input tidak boleh bernilai negatif untuk operasi akar kuadrat

class Calculator:
    def __init__(self):
        self.history = []

    def add(self, a, b):
        result = a + b
        self.record_operation("add", result)
        return result

    def record_operation(self, op_name, value):
        self.history.append((op_name, value))
