import I2C_LCD_driver, time

lcd = I2C_LCD_driver.lcd()
lcd.display_string('hola')

time.sleep(3)
lcd.lcd_clear()
